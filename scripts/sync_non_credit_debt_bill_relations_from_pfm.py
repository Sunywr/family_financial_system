from __future__ import annotations

import argparse
import os
from dataclasses import dataclass
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
class SyncStats:
    pfm_rows_with_debt: int = 0
    ffs_bill_exists: int = 0
    ffs_non_credit_debt_exists: int = 0
    mapped_by_source_bill: int = 0
    unresolved_non_credit_debt: int = 0
    already_linked: int = 0
    would_update: int = 0
    updated: int = 0


def fetch_all(conn: pymysql.Connection, sql: str, params: tuple[Any, ...] | None = None) -> list[dict[str, Any]]:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())
        return list(cur.fetchall())


def fetch_one(conn: pymysql.Connection, sql: str, params: tuple[Any, ...] | None = None) -> dict[str, Any] | None:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())
        return cur.fetchone()


def sync_relations(apply_changes: bool) -> SyncStats:
    stats = SyncStats()

    pfm = pymysql.connect(**PFM_DB)
    ffs = pymysql.connect(**FFS_DB)
    try:
        pfm_rows = fetch_all(
            pfm,
            """
            SELECT a.id AS account_id, a.owner_id_id AS user_id, a.debt_id, dc.name AS debt_category_name
            FROM account_account a
            JOIN account_debt d ON d.id = a.debt_id
            JOIN account_debtcategory dc ON dc.id = d.debt_type_id
            WHERE a.debt_id IS NOT NULL
            ORDER BY a.id ASC
            """,
        )
        stats.pfm_rows_with_debt = len(pfm_rows)

        # For remapped imports, non-credit FFS debt ID may differ from PFM debt_id.
        # Use the earliest source bill id per (user_id, debt_id) as a stable bridge key.
        source_bill_by_debt: dict[tuple[int, int], int] = {}
        for row in pfm_rows:
            if row.get("debt_category_name") == "credit_card":
                continue
            key = (int(row["user_id"]), int(row["debt_id"]))
            bill_id = int(row["account_id"])
            existing = source_bill_by_debt.get(key)
            if existing is None or bill_id < existing:
                source_bill_by_debt[key] = bill_id

        resolved_debt_cache: dict[tuple[int, int], int | None] = {}
        updates: list[tuple[int, int, int]] = []
        # tuple: (resolved_debt_id, bill_id, user_id)
        for row in pfm_rows:
            bill_id = int(row["account_id"])
            user_id = int(row["user_id"])
            debt_id = int(row["debt_id"])
            debt_category_name = str(row.get("debt_category_name") or "")

            # We only repair non-credit debt relations.
            if debt_category_name == "credit_card":
                continue

            bill = fetch_one(
                ffs,
                """
                SELECT id, user_id, related_debt_id
                FROM bills
                WHERE id = %s AND user_id = %s AND deleted_at IS NULL
                """,
                (bill_id, user_id),
            )
            if not bill:
                continue
            stats.ffs_bill_exists += 1

            cache_key = (user_id, debt_id)
            resolved_debt_id = resolved_debt_cache.get(cache_key)
            if resolved_debt_id is None and cache_key not in resolved_debt_cache:
                direct = fetch_one(
                    ffs,
                    """
                    SELECT id
                    FROM debts
                    WHERE id = %s
                      AND user_id = %s
                      AND payment_method <> 'credit_card'
                      AND deleted_at IS NULL
                    """,
                    (debt_id, user_id),
                )
                if direct:
                    resolved_debt_id = int(direct["id"])
                else:
                    source_bill_id = source_bill_by_debt.get(cache_key)
                    remapped = (
                        fetch_one(
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
                        if source_bill_id is not None
                        else None
                    )
                    if remapped:
                        resolved_debt_id = int(remapped["id"])
                        stats.mapped_by_source_bill += 1
                resolved_debt_cache[cache_key] = resolved_debt_id

            if resolved_debt_id is None:
                stats.unresolved_non_credit_debt += 1
                continue

            stats.ffs_non_credit_debt_exists += 1

            current_related = bill.get("related_debt_id")
            if current_related is not None and int(current_related) == resolved_debt_id:
                stats.already_linked += 1
                continue

            updates.append((resolved_debt_id, bill_id, user_id))

        stats.would_update = len(updates)

        if apply_changes and updates:
            with ffs.cursor() as cur:
                cur.executemany(
                    """
                    UPDATE bills
                    SET related_debt_id = %s,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                      AND user_id = %s
                      AND deleted_at IS NULL
                    """,
                    updates,
                )
                stats.updated = cur.rowcount
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
        description="Sync missing non-credit debt bill relations in FFS from PFM account_account.debt_id"
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Actually write updates to FFS bills.related_debt_id (default is dry-run)",
    )
    args = parser.parse_args()

    stats = sync_relations(apply_changes=args.apply)
    print("[sync_non_credit_debt_bill_relations_from_pfm]")
    print(f"- pfm_rows_with_debt: {stats.pfm_rows_with_debt}")
    print(f"- ffs_bill_exists: {stats.ffs_bill_exists}")
    print(f"- ffs_non_credit_debt_exists: {stats.ffs_non_credit_debt_exists}")
    print(f"- mapped_by_source_bill: {stats.mapped_by_source_bill}")
    print(f"- unresolved_non_credit_debt: {stats.unresolved_non_credit_debt}")
    print(f"- already_linked: {stats.already_linked}")
    print(f"- would_update: {stats.would_update}")
    if args.apply:
        print(f"- updated: {stats.updated}")
    else:
        print("- mode: dry-run (no data changed)")


if __name__ == "__main__":
    main()
