from __future__ import annotations

import argparse
import os
from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal
from typing import Any

import pymysql
from pymysql.cursors import DictCursor


PFM_DB = {
    "host": os.environ.get("PFM_DB_HOST", "127.0.0.1"),
    "port": int(os.environ.get("PFM_DB_PORT", "45106")),
    "user": os.environ.get("PFM_DB_USER", "pfm"),
    "password": os.environ.get("PFM_DB_PASSWORD", "sywr0830"),
    "database": os.environ.get("PFM_DB_NAME", "pfm"),
    "charset": "utf8mb4",
    "cursorclass": DictCursor,
    "autocommit": True,
}

FFS_DB = {
    "host": os.environ.get("FFS_DB_HOST", "127.0.0.1"),
    "port": int(os.environ.get("FFS_DB_PORT", "3306")),
    "user": os.environ.get("FFS_DB_USER", "ffs"),
    "password": os.environ.get("FFS_DB_PASSWORD", "sywr0830"),
    "database": os.environ.get("FFS_DB_NAME", "hfs"),
    "charset": "utf8mb4",
    "cursorclass": DictCursor,
    "autocommit": False,
}


@dataclass
class RepairStats:
    templates_in_pfm: int = 0
    templates_with_bills: int = 0
    reused_templates: int = 0
    created_templates: int = 0
    missing_category: int = 0
    linked_bills: int = 0
    already_linked_bills: int = 0
    missing_ffs_bill: int = 0
    generated_candidates: int = 0
    generated_soft_deleted: int = 0


def fetch_all(conn: pymysql.Connection, sql: str, params: tuple[Any, ...] | None = None) -> list[dict[str, Any]]:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())
        return list(cur.fetchall())


def fetch_one(conn: pymysql.Connection, sql: str, params: tuple[Any, ...] | None = None) -> dict[str, Any] | None:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())
        return cur.fetchone()


def to_date(value: Any) -> date:
    if value is None:
        return date.today()
    if isinstance(value, date):
        return value
    if isinstance(value, datetime):
        return value.date()
    return datetime.fromisoformat(str(value)).date()


def month_span_inclusive(start: date, end: date) -> int:
    if end < start:
        return 1
    return (end.year - start.year) * 12 + (end.month - start.month) + 1


def build_remark(description: str | None, remark: str | None, pfm_debt_id: int) -> str:
    left = (description or "").strip()
    right = (remark or "").strip()
    if left and right:
        return f"{left} | {right} | pfm_template#{pfm_debt_id}"
    if left:
        return f"{left} | pfm_template#{pfm_debt_id}"
    if right:
        return f"{right} | pfm_template#{pfm_debt_id}"
    return f"pfm_template#{pfm_debt_id}"


def repair(apply_changes: bool, cleanup_generated: bool) -> RepairStats:
    stats = RepairStats()

    pfm = pymysql.connect(**PFM_DB)
    ffs = pymysql.connect(**FFS_DB)
    try:
        category_rows = fetch_all(
            ffs,
            """
            SELECT id, name
            FROM config_items
            WHERE config_type = 'debt_category'
            """,
        )
        category_id_by_name = {str(row["name"]): int(row["id"]) for row in category_rows}

        templates = fetch_all(
            pfm,
            """
            SELECT d.id AS pfm_debt_id,
                   d.owner_id_id AS user_id,
                   d.debt_amount,
                   d.repayment_at,
                   d.repayment_deadline,
                   d.repayment_cycle,
                   d.status,
                   d.description,
                   d.remark,
                   dc.name AS category_name,
                   MIN(a.id) AS source_bill_id,
                   COUNT(a.id) AS bill_count
            FROM account_debt d
            JOIN account_debtcategory dc
              ON dc.id = d.debt_type_id
            LEFT JOIN account_account a
              ON a.debt_id = d.id
             AND a.owner_id_id = d.owner_id_id
            WHERE dc.name <> 'credit_card'
              AND d.source_debt_id IS NULL
            GROUP BY d.id,
                     d.owner_id_id,
                     d.debt_amount,
                     d.repayment_at,
                     d.repayment_deadline,
                     d.repayment_cycle,
                     d.status,
                     d.description,
                     d.remark,
                     dc.name
            ORDER BY d.id ASC
            """,
        )
        stats.templates_in_pfm = len(templates)

        template_map: dict[tuple[int, int], int] = {}

        for row in templates:
            pfm_debt_id = int(row["pfm_debt_id"])
            user_id = int(row["user_id"])
            source_bill_id = row.get("source_bill_id")
            bill_count = int(row.get("bill_count") or 0)
            category_name = str(row.get("category_name") or "").strip()

            if bill_count <= 0 or source_bill_id is None:
                continue
            stats.templates_with_bills += 1

            category_id = category_id_by_name.get(category_name)
            if category_id is None:
                stats.missing_category += 1
                continue

            source_bill_id = int(source_bill_id)
            existing = fetch_one(
                ffs,
                """
                SELECT id
                FROM debts
                WHERE user_id = %s
                  AND source_bill_id = %s
                  AND payment_method <> 'credit_card'
                  AND deleted_at IS NULL
                ORDER BY id ASC
                LIMIT 1
                """,
                (user_id, source_bill_id),
            )

            if existing:
                resolved_ffs_debt_id = int(existing["id"])
                stats.reused_templates += 1
            else:
                start_date = to_date(row.get("repayment_at"))
                end_date = to_date(row.get("repayment_deadline"))
                period_count = month_span_inclusive(start_date, end_date)
                period_value = int(row.get("repayment_cycle") or 1)
                if period_value <= 0:
                    period_value = 1
                debt_amount = Decimal(str(row.get("debt_amount") or 0))
                debt_status = "settled" if int(row.get("status") or 0) == 1 else "pending"
                debt_remark = build_remark(row.get("description"), row.get("remark"), pfm_debt_id)

                if apply_changes:
                    with ffs.cursor() as cur:
                        cur.execute(
                            """
                            INSERT INTO debts (
                                user_id,
                                source_bill_id,
                                start_date,
                                end_date,
                                repay_deadline,
                                category_id,
                                category_name,
                                amount,
                                period_count,
                                period_unit,
                                period_value,
                                payment_method,
                                status,
                                remark,
                                created_at,
                                updated_at
                            ) VALUES (
                                %s, %s, %s, %s, %s,
                                %s, %s, %s, %s, %s,
                                %s, %s, %s, %s,
                                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                            )
                            """,
                            (
                                user_id,
                                source_bill_id,
                                start_date,
                                end_date,
                                end_date,
                                category_id,
                                category_name,
                                debt_amount,
                                period_count,
                                "month",
                                period_value,
                                "cash",
                                debt_status,
                                debt_remark,
                            ),
                        )
                        resolved_ffs_debt_id = int(cur.lastrowid)
                else:
                    # Dry-run placeholder; distinct from real IDs.
                    resolved_ffs_debt_id = -pfm_debt_id

                stats.created_templates += 1

            template_map[(user_id, pfm_debt_id)] = resolved_ffs_debt_id

        bill_rows = fetch_all(
            pfm,
            """
            SELECT a.id AS bill_id,
                   a.owner_id_id AS user_id,
                   a.debt_id AS pfm_debt_id
            FROM account_account a
            JOIN account_debt d ON d.id = a.debt_id
            JOIN account_debtcategory dc ON dc.id = d.debt_type_id
            WHERE a.debt_id IS NOT NULL
              AND dc.name <> 'credit_card'
            ORDER BY a.id ASC
            """,
        )

        for row in bill_rows:
            bill_id = int(row["bill_id"])
            user_id = int(row["user_id"])
            pfm_debt_id = int(row["pfm_debt_id"])

            resolved_ffs_debt_id = template_map.get((user_id, pfm_debt_id))
            if resolved_ffs_debt_id is None:
                continue

            ffs_bill = fetch_one(
                ffs,
                """
                SELECT id, related_debt_id
                FROM bills
                WHERE id = %s
                  AND user_id = %s
                  AND deleted_at IS NULL
                """,
                (bill_id, user_id),
            )
            if not ffs_bill:
                stats.missing_ffs_bill += 1
                continue

            current_related = ffs_bill.get("related_debt_id")
            if current_related is not None and int(current_related) == resolved_ffs_debt_id:
                stats.already_linked_bills += 1
                continue

            stats.linked_bills += 1
            if apply_changes:
                with ffs.cursor() as cur:
                    cur.execute(
                        """
                        UPDATE bills
                        SET related_debt_id = %s,
                            updated_at = CURRENT_TIMESTAMP
                        WHERE id = %s
                          AND user_id = %s
                          AND deleted_at IS NULL
                        """,
                        (resolved_ffs_debt_id, bill_id, user_id),
                    )

        if cleanup_generated:
            generated_rows = fetch_all(
                pfm,
                """
                SELECT id AS generated_debt_id, owner_id_id AS user_id
                FROM account_debt
                WHERE source_debt_id IS NOT NULL
                """,
            )
            stats.generated_candidates = len(generated_rows)
            if apply_changes and generated_rows:
                with ffs.cursor() as cur:
                    for row in generated_rows:
                        cur.execute(
                            """
                            UPDATE debts
                            SET deleted_at = CURRENT_TIMESTAMP,
                                updated_at = CURRENT_TIMESTAMP
                            WHERE id = %s
                              AND user_id = %s
                              AND payment_method <> 'credit_card'
                              AND deleted_at IS NULL
                            """,
                            (int(row["generated_debt_id"]), int(row["user_id"])),
                        )
                        stats.generated_soft_deleted += cur.rowcount

        if apply_changes:
            ffs.commit()
        else:
            ffs.rollback()

        return stats
    except Exception:
        ffs.rollback()
        raise
    finally:
        pfm.close()
        ffs.close()


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Repair non-credit debt templates and bill relations in FFS from PFM data"
    )
    parser.add_argument("--apply", action="store_true", help="Write changes to FFS (default: dry-run)")
    parser.add_argument(
        "--cleanup-generated",
        action="store_true",
        help="Soft-delete generated non-credit debts imported from PFM source_debt_id rows",
    )
    args = parser.parse_args()

    stats = repair(apply_changes=args.apply, cleanup_generated=args.cleanup_generated)
    print("[repair_non_credit_debt_templates_from_pfm]")
    print(f"- templates_in_pfm: {stats.templates_in_pfm}")
    print(f"- templates_with_bills: {stats.templates_with_bills}")
    print(f"- reused_templates: {stats.reused_templates}")
    print(f"- created_templates: {stats.created_templates}")
    print(f"- missing_category: {stats.missing_category}")
    print(f"- linked_bills: {stats.linked_bills}")
    print(f"- already_linked_bills: {stats.already_linked_bills}")
    print(f"- missing_ffs_bill: {stats.missing_ffs_bill}")
    print(f"- generated_candidates: {stats.generated_candidates}")
    if args.apply:
        print(f"- generated_soft_deleted: {stats.generated_soft_deleted}")
    else:
        print("- mode: dry-run (no data changed)")


if __name__ == "__main__":
    main()
