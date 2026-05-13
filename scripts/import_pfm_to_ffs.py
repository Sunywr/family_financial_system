from __future__ import annotations

import hashlib
import json
from collections import defaultdict
from datetime import date, datetime
from decimal import Decimal
from typing import Any

import pymysql
from pymysql.cursors import DictCursor


SOURCE_DB = {
    "host": "127.0.0.1",
    "port": 3306,
    "user": "pfm",
    "password": "sywr0830",
    "database": "pfm",
    "charset": "utf8mb4",
    "cursorclass": DictCursor,
    "autocommit": True,
}

TARGET_DB = {
    "host": "127.0.0.1",
    "port": 3306,
    "user": "ffs",
    "password": "123456",
    "database": "ffs",
    "charset": "utf8mb4",
    "cursorclass": DictCursor,
    "autocommit": False,
}

PASSWORDS = {
    "admin": "sywr0830",
    "孙意蔚然": "sywr0830",
    "刘洁": "liujie123",
}


def sha256_password(raw: str) -> str:
    return "sha256$" + hashlib.sha256(raw.encode("utf-8")).hexdigest()


def dec2(value: Any) -> str:
    if value is None:
        return "0.00"
    return str(Decimal(str(value)).quantize(Decimal("0.01")))


def dec6(value: Any) -> str:
    if value is None:
        return "0.000000"
    return str(Decimal(str(value)).quantize(Decimal("0.000001")))


def combine_text(*parts: Any) -> str | None:
    values = []
    for part in parts:
        if part is None:
            continue
        text = str(part).replace("\r", " ").replace("\n", " / ").strip()
        if text:
            values.append(text)
    return " | ".join(values) if values else None


def truncate_text(value: Any, limit: int = 255) -> str | None:
    if value is None:
        return None
    text = str(value)
    if len(text) <= limit:
        return text
    return text[: max(0, limit - 3)] + "..."


def first_day(value: datetime | date | None) -> date | None:
    if value is None:
        return None
    if isinstance(value, datetime):
        value = value.date()
    return value.replace(day=1)


def map_account_category_name(name: str) -> str:
    lowered = name.lower()
    if lowered == "financial":
        return "wealth"
    if lowered == "stock":
        return "stock"
    return lowered


def map_debt_category_name(name: str) -> str:
    lowered = name.lower()
    if lowered in {"credit_card"}:
        return "credit_card"
    if lowered in {"loan"}:
        return "loan"
    if lowered in {"installments", "installment"}:
        return "installment"
    return lowered


def map_investment_type(name: str | None, display_name: str | None) -> str:
    raw = (name or display_name or "").lower()
    if raw == "stock" or display_name == "股票":
        return "stock"
    return "wealth"


def map_brand_board(level: str | None, score: Any) -> str:
    if level == "A":
        return "red"
    numeric = float(score or 0)
    if numeric <= 2:
        return "black"
    return "normal"


def map_debt_payment_method(debt_row: dict[str, Any], debt_category_name: str, description: str) -> str:
    if (
        debt_category_name == "credit_card"
        or "信用卡" in description
        or "白条" in description
        or "花呗" in description
        or "月付" in description
        or debt_row.get("credit_card_id") is not None
    ):
        return "credit_card"
    return "cash"


def map_presale_status(status: int) -> str:
    if status == 1:
        return "completed"
    if status == 3:
        return "cancelled"
    return "pending_final_payment"


def map_debt_status(status: int) -> str:
    return "settled" if status == 1 else "pending"


def map_asset_status(status: int) -> str:
    return "active" if status == 0 else "archived"


def role_for_user(row: dict[str, Any], perm_count: int) -> str:
    if row["username"] == "admin" or row["is_superuser"] or perm_count > 0:
        return "owner"
    return "member"


def fetch_all(conn, sql: str, params: tuple[Any, ...] | None = None) -> list[dict[str, Any]]:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())
        return list(cur.fetchall())


def fetch_one(conn, sql: str, params: tuple[Any, ...] | None = None) -> dict[str, Any] | None:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())
        return cur.fetchone()


def execute_many(conn, sql: str, rows: list[tuple[Any, ...]]) -> None:
    if not rows:
        return
    with conn.cursor() as cur:
        cur.executemany(sql, rows)


def execute(conn, sql: str, params: tuple[Any, ...] | None = None) -> None:
    with conn.cursor() as cur:
        cur.execute(sql, params or ())


def ensure_config_item(
    conn,
    *,
    config_type: str,
    name: str,
    display_name: str,
    is_builtin: bool = False,
    enabled: bool = True,
    sort_order: int = 0,
) -> int:
    existing = fetch_one(
        conn,
        "SELECT id FROM config_items WHERE config_type = %s AND name = %s AND deleted_at IS NULL",
        (config_type, name),
    )
    if existing:
        execute(
            conn,
            "UPDATE config_items SET display_name = %s, enabled = %s, sort_order = %s WHERE id = %s",
            (display_name, enabled, sort_order, existing["id"]),
        )
        return int(existing["id"])
    with conn.cursor() as cur:
        cur.execute(
            """
            INSERT INTO config_items (config_type, name, display_name, is_builtin, enabled, sort_order)
            VALUES (%s, %s, %s, %s, %s, %s)
            """,
            (config_type, name, display_name, 1 if is_builtin else 0, 1 if enabled else 0, sort_order),
        )
        return int(cur.lastrowid)


def import_users(src, dst) -> dict[int, str]:
    users = fetch_all(
        src,
        """
        SELECT u.id, u.username, u.first_name, u.last_name, u.is_superuser, u.is_staff, u.is_active, u.date_joined
        FROM auth_user u
        ORDER BY u.id
        """,
    )
    group_rows = fetch_all(
        src,
        """
        SELECT ug.user_id, g.name
        FROM auth_user_groups ug
        JOIN auth_group g ON g.id = ug.group_id
        """,
    )
    perm_rows = fetch_all(
        src,
        """
        SELECT user_id, COUNT(*) AS perm_count
        FROM auth_user_user_permissions
        GROUP BY user_id
        """,
    )
    groups_by_user: dict[int, list[str]] = defaultdict(list)
    for row in group_rows:
        groups_by_user[int(row["user_id"])].append(row["name"])
    perms_by_user = {int(row["user_id"]): int(row["perm_count"]) for row in perm_rows}

    rows = []
    role_by_user: dict[int, str] = {}
    for row in users:
        username = row["username"]
        role = role_for_user(row, perms_by_user.get(int(row["id"]), 0))
        role_by_user[int(row["id"])] = role
        display_name = username if username != "admin" else "admin"
        raw_password = PASSWORDS[username]
        rows.append(
            (
                int(row["id"]),
                username,
                display_name,
                sha256_password(raw_password),
                role,
                1 if row["is_active"] else 0,
                row["date_joined"],
                row["date_joined"],
            )
        )

    execute_many(
        dst,
        """
        INSERT INTO users (id, username, display_name, password_hash, role, enabled, created_at, updated_at)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        """,
        rows,
    )

    intel_rows = []
    for row in users:
        user_id = int(row["id"])
        intel_rows.append(
            (
                user_id,
                f"Legacy permission snapshot: {row['username']}",
                "pfm_auth",
                date.today(),
                "archived",
                "permissions,legacy",
                f"role={role_by_user[user_id]}, groups={','.join(groups_by_user[user_id]) or 'none'}, direct_perms={perms_by_user.get(user_id, 0)}",
                json.dumps(
                    {
                        "username": row["username"],
                        "is_superuser": bool(row["is_superuser"]),
                        "is_staff": bool(row["is_staff"]),
                        "groups": groups_by_user[user_id],
                        "direct_perm_count": perms_by_user.get(user_id, 0),
                    },
                    ensure_ascii=False,
                ),
            )
        )

    execute_many(
        dst,
        """
        INSERT INTO intel_items (user_id, title, source, item_date, status, tags, summary, content)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        """,
        intel_rows,
    )
    return role_by_user


def import_config_items(src, dst) -> dict[str, dict[int, tuple[int, str]]]:
    category_maps: dict[str, dict[int, tuple[int, str]]] = {}

    account_categories = fetch_all(src, "SELECT id, name, display_name FROM account_category ORDER BY id")
    account_map: dict[int, tuple[int, str]] = {}
    for row in account_categories:
        new_name = map_account_category_name(row["name"])
        new_id = ensure_config_item(
            dst,
            config_type="account_category",
            name=new_name,
            display_name=row["display_name"] or row["name"],
            is_builtin=new_name in {"stock", "wealth"},
            sort_order=int(row["id"]),
        )
        account_map[int(row["id"])] = (new_id, row["display_name"] or row["name"])
    category_maps["account"] = account_map

    debt_categories = fetch_all(src, "SELECT id, name, display_name FROM account_debtcategory ORDER BY id")
    debt_map: dict[int, tuple[int, str]] = {}
    for row in debt_categories:
        new_name = map_debt_category_name(row["name"])
        new_id = ensure_config_item(
            dst,
            config_type="debt_category",
            name=new_name,
            display_name=row["display_name"] or row["name"],
            is_builtin=new_name in {"credit_card", "loan", "installment"},
            sort_order=int(row["id"]),
        )
        debt_map[int(row["id"])] = (new_id, row["display_name"] or row["name"])
    category_maps["debt"] = debt_map

    presale_map: dict[int, tuple[int, str]] = {}
    for row in fetch_all(src, "SELECT id, name, display_name FROM account_presalecategory ORDER BY id"):
        new_id = ensure_config_item(
            dst,
            config_type="presale_category",
            name=row["name"].lower(),
            display_name=row["display_name"] or row["name"],
            sort_order=int(row["id"]),
        )
        presale_map[int(row["id"])] = (new_id, row["display_name"] or row["name"])
    category_maps["presale"] = presale_map

    asset_map: dict[int, tuple[int, str]] = {}
    for row in fetch_all(src, "SELECT id, name, display_name FROM account_assetcategory ORDER BY id"):
        new_id = ensure_config_item(
            dst,
            config_type="asset_category",
            name=row["name"].lower(),
            display_name=row["display_name"] or row["name"],
            sort_order=int(row["id"]),
        )
        asset_map[int(row["id"])] = (new_id, row["display_name"] or row["name"])
    category_maps["asset"] = asset_map

    budget_item_map: dict[int, tuple[int, str]] = {}
    for row in fetch_all(src, "SELECT id, name, display_name FROM account_budgetcategory ORDER BY id"):
        display_name = row["display_name"] or row["name"]
        account_match = fetch_one(
            dst,
            """
            SELECT id, display_name
            FROM config_items
            WHERE config_type = 'account_category' AND display_name = %s AND deleted_at IS NULL
            ORDER BY id LIMIT 1
            """,
            (display_name,),
        )
        if account_match:
            budget_item_map[int(row["id"])] = (int(account_match["id"]), account_match["display_name"])
            continue
        new_id = ensure_config_item(
            dst,
            config_type="budget_category",
            name=row["name"].lower(),
            display_name=display_name,
            sort_order=int(row["id"]),
        )
        budget_item_map[int(row["id"])] = (new_id, display_name)
    category_maps["budget"] = budget_item_map

    brand_default = fetch_one(
        dst,
        """
        SELECT id, display_name FROM config_items
        WHERE config_type = 'brand_category' AND deleted_at IS NULL
        ORDER BY is_builtin DESC, id ASC LIMIT 1
        """,
    )
    if brand_default:
        category_maps["brand"] = {1: (int(brand_default["id"]), brand_default["display_name"])}

    for row in fetch_all(src, "SELECT id, name, display_name FROM account_investcategory ORDER BY id"):
        ensure_config_item(
            dst,
            config_type="investment_category",
            name=row["name"].lower(),
            display_name=row["display_name"] or row["name"],
            sort_order=int(row["id"]),
        )

    return category_maps


def import_credit_cards(src, dst) -> None:
    rows = fetch_all(src, "SELECT * FROM account_creditcard ORDER BY id")
    payload = []
    for row in rows:
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"]),
                row["description"],
                int(row["create_at"].day),
                int(row["repayment_at"].day),
                "0.00",
                1 if int(row["status"]) == 0 else 0,
                row["create_at"],
                row["repayment_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO credit_cards (
            id, user_id, name, billing_day, repayment_day, credit_limit, enabled, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_debts(src, dst, category_maps: dict[str, dict[int, tuple[int, str]]]) -> dict[int, str]:
    debt_rows = fetch_all(src, "SELECT * FROM account_debt ORDER BY id")
    debt_categories = {int(row["id"]): row for row in fetch_all(src, "SELECT id, name, display_name FROM account_debtcategory")}
    source_bill_rows = fetch_all(
        src,
        "SELECT debt_id, MIN(id) AS bill_id FROM account_account WHERE debt_id IS NOT NULL GROUP BY debt_id",
    )
    source_bill_map = {int(row["debt_id"]): int(row["bill_id"]) for row in source_bill_rows}
    debt_category_name_by_debt_id: dict[int, str] = {}
    payload = []
    for row in debt_rows:
        old_category = debt_categories[int(row["debt_type_id"])]
        new_category_id, new_category_name = category_maps["debt"][int(row["debt_type_id"])]
        debt_category_name_by_debt_id[int(row["id"])] = map_debt_category_name(old_category["name"])
        description = row["description"] or new_category_name
        payment_method = map_debt_payment_method(row, map_debt_category_name(old_category["name"]), description)
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"]),
                source_bill_map.get(int(row["id"])),
                row["create_at"].date(),
                row["repayment_at"].date() if row["repayment_at"] else None,
                row["repayment_deadline"].date() if row["repayment_deadline"] else None,
                new_category_id,
                new_category_name,
                dec2(row["debt_amount"]),
                max(int(row["repayment_cycle"] or 1), 1),
                "month",
                max(int(row["repayment_cycle"] or 1), 1),
                payment_method,
                map_debt_status(int(row["status"])),
                combine_text(row["description"], row["remark"]),
                row["create_at"],
                row["repayment_at"] or row["create_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO debts (
            id, user_id, source_bill_id, start_date, end_date, repay_deadline, category_id, category_name,
            amount, period_count, period_unit, period_value, payment_method, status, remark, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )
    return debt_category_name_by_debt_id


def import_presales(src, dst, category_maps: dict[str, dict[int, tuple[int, str]]]) -> None:
    rows = fetch_all(src, "SELECT * FROM account_presale ORDER BY id")
    payload = []
    for row in rows:
        new_category_id, new_category_name = category_maps["presale"][int(row["presale_type_id"])]
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"]),
                None,
                row["create_at"].date(),
                row["balance_payment_at"].date() if row["balance_payment_at"] else None,
                new_category_id,
                new_category_name,
                dec2(row["deposit"]),
                dec2(row["balance_payment"]),
                map_presale_status(int(row["status"])),
                combine_text(row["description"], row["remark"]),
                row["create_at"],
                row["balance_payment_at"] or row["create_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO presales (
            id, user_id, source_bill_id, deposit_date, final_payment_date, category_id, category_name,
            deposit_amount, final_payment_amount, status, remark, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_assets(src, dst, category_maps: dict[str, dict[int, tuple[int, str]]]) -> None:
    rows = fetch_all(src, "SELECT * FROM account_asset ORDER BY id")
    source_bill_rows = fetch_all(
        src,
        "SELECT asset_id, MIN(id) AS bill_id FROM account_account WHERE asset_id IS NOT NULL GROUP BY asset_id",
    )
    source_bill_map = {int(row["asset_id"]): int(row["bill_id"]) for row in source_bill_rows}
    payload = []
    for row in rows:
        category_id, category_name = category_maps["asset"][int(row["asset_type_id"])]
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"]),
                source_bill_map.get(int(row["id"])),
                row["description"],
                category_id,
                category_name,
                dec2(row["value"]),
                combine_text(row["description"], row["remark"]),
                map_asset_status(int(row["status"])),
                row["create_at"],
                row["update_at"] or row["create_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO assets (
            id, user_id, source_bill_id, name, category_id, category_name, amount, remark, status, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_investments(src, dst) -> dict[int, dict[str, Any]]:
    invest_rows = fetch_all(src, "SELECT * FROM account_invest ORDER BY id")
    invest_categories = {int(row["id"]): row for row in fetch_all(src, "SELECT id, name, display_name FROM account_investcategory")}
    source_bill_rows = fetch_all(
        src,
        "SELECT invest_id, MIN(id) AS bill_id FROM account_account WHERE invest_id IS NOT NULL GROUP BY invest_id",
    )
    source_bill_map = {int(row["invest_id"]): int(row["bill_id"]) for row in source_bill_rows}
    invest_info: dict[int, dict[str, Any]] = {}
    seen_codes: set[tuple[int, str, str]] = set()
    payload = []
    for row in invest_rows:
        invest_type_row = invest_categories[int(row["invest_type_id"])]
        investment_type = map_investment_type(invest_type_row["name"], invest_type_row["display_name"])
        raw_code = str(row["code"] or "").strip()
        code = raw_code if raw_code and raw_code != "0" else f"legacy_{row['id']}"
        code_key = (int(row["owner_id_id"]), investment_type, code)
        if code_key in seen_codes:
            code = f"{code}_{row['id']}"
            code_key = (int(row["owner_id_id"]), investment_type, code)
        seen_codes.add(code_key)
        cost_value = Decimal(str(row["cost"] or 0))
        profit_value = Decimal(str(row["profit"] or 0))
        share_value = Decimal(str(row["share"] or 0))
        average_cost = Decimal(str(row["price"] or 0))
        if share_value > 0 and cost_value > 0:
            average_cost = cost_value / share_value
        total_profit_rate = Decimal("0")
        if cost_value > 0:
            total_profit_rate = profit_value / cost_value
        status = "sold" if int(row["status"]) == 1 and share_value == 0 else "holding"
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"]),
                source_bill_map.get(int(row["id"])),
                investment_type,
                row["description"],
                code,
                row["description"],
                None,
                dec6(row["share"]),
                dec2(row["cost"]),
                dec6(average_cost),
                dec6(row["price"]),
                dec2(row["value"]),
                dec2(profit_value if status == "sold" else 0),
                dec2(profit_value if status != "sold" else 0),
                dec2(row["profit"]),
                dec6(total_profit_rate),
                status,
                row["create_at"],
                row["update_at"] or row["create_at"],
            )
        )
        invest_info[int(row["id"])] = {
            "investment_type": investment_type,
            "description": row["description"],
            "code": code,
            "price": row["price"],
        }
    execute_many(
        dst,
        """
        INSERT INTO investments (
            id, user_id, source_bill_id, investment_type, name, code, organization_name, market,
            total_shares, total_cost, average_cost, current_price, market_value, realized_profit,
            unrealized_profit, total_profit, total_profit_rate, status, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )
    return invest_info


def import_budgets(src, dst, category_maps: dict[str, dict[int, tuple[int, str]]]) -> None:
    rows = fetch_all(src, "SELECT * FROM account_budget ORDER BY id")
    aggregated: dict[tuple[int, date, int], dict[str, Any]] = {}
    for row in rows:
        category_id, category_name = category_maps["budget"][int(row["budget_type_id"])]
        budget_month = first_day(
            datetime.strptime(f"{row['month_key']}-01", "%Y-%m-%d").date()
            if row["month_key"]
            else row["create_at"]
        )
        description = row["description"] or ""
        key = (int(row["owner_id_id"]), budget_month, category_id)
        budget_value = Decimal(str(row["budget"] or 0))
        manual_adjusted = 0 if "环比生成" in description and budget_value == 0 else 1
        remark = combine_text(row["description"], row["remark"])
        existing = aggregated.get(key)
        if existing is None:
            aggregated[key] = {
                "id": int(row["id"]),
                "user_id": int(row["owner_id_id"]),
                "budget_month": budget_month,
                "category_id": category_id,
                "category_name": category_name,
                "planned_amount": budget_value,
                "manual_adjusted": manual_adjusted,
                "remarks": [remark] if remark else [],
                "created_at": row["create_at"],
                "updated_at": row["updated_at"] if "updated_at" in row else row["create_at"],
            }
            continue

        existing["planned_amount"] += budget_value
        existing["manual_adjusted"] = max(existing["manual_adjusted"], manual_adjusted)
        if remark and remark not in existing["remarks"]:
            existing["remarks"].append(remark)
        existing["created_at"] = min(existing["created_at"], row["create_at"])
        existing["updated_at"] = max(
            existing["updated_at"],
            row["updated_at"] if "updated_at" in row else row["create_at"],
        )

    payload = []
    for item in aggregated.values():
        payload.append(
            (
                item["id"],
                item["user_id"],
                item["budget_month"],
                item["category_id"],
                item["category_name"],
                dec2(item["planned_amount"]),
                item["manual_adjusted"],
                " || ".join(item["remarks"]) if item["remarks"] else None,
                item["created_at"],
                item["updated_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO budgets (
            id, user_id, budget_month, category_id, category_name, planned_amount, manual_adjusted, remark, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_brands(src, dst, category_maps: dict[str, dict[int, tuple[int, str]]]) -> None:
    rows = fetch_all(src, "SELECT * FROM account_brand ORDER BY id")
    category_id, category_name = next(iter(category_maps["brand"].values()))
    payload = []
    for row in rows:
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"] or 1),
                category_id,
                category_name,
                row["name"],
                int(round(float(row["score"] or 0) * 20)),
                map_brand_board(row["level"], row["score"]),
                row["review"],
                combine_text(row["merchant_name"], row["level"]),
                row["created_at"] or datetime.now(),
                row["updated_at"] or row["created_at"] or datetime.now(),
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO brands (
            id, user_id, category_id, category_name, brand_name, score, board_type, review, remark, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_balance_calibrations(src, dst) -> None:
    rows = fetch_all(src, "SELECT * FROM account_balancefix ORDER BY id")
    payload = []
    for row in rows:
        payload.append(
            (
                int(row["id"]),
                int(row["owner_id_id"]),
                row["created_at"].date(),
                dec2(row["balance"]),
                "migrated from pfm balancefix",
                row["created_at"],
                row["created_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO balance_calibrations (
            id, user_id, calibration_date, cash_balance, remark, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_bills_and_transactions(src, dst, category_maps, debt_category_by_id, invest_info) -> None:
    accounts = fetch_all(src, "SELECT * FROM account_account ORDER BY account_date, id")
    labels = fetch_all(
        src,
        """
        SELECT aal.account_id_id AS account_id, l.name
        FROM account_accountlabel aal
        JOIN account_label l ON l.id = aal.label_id_id
        ORDER BY aal.account_id_id, l.name
        """,
    )
    tags_by_account: dict[int, list[str]] = defaultdict(list)
    for row in labels:
        tags_by_account[int(row["account_id"])].append(row["name"])

    invest_sequence: dict[int, int] = defaultdict(int)
    bill_rows = []
    tx_rows = []
    for row in accounts:
        account_id = int(row["id"])
        category_id, category_name = category_maps["account"][int(row["account_type_id"])]
        invest_id = int(row["invest_id"]) if row["invest_id"] is not None else None
        if invest_id is not None:
            invest_sequence[invest_id] += 1
        amount_source = row["expenditure"] if row["expenditure"] not in (None, 0) else row["income"]
        amount = dec2(amount_source or 0)
        if row["refund_of_id"] is not None:
            bill_type = "refund"
        elif row["income"] not in (None, 0):
            bill_type = "income"
        else:
            bill_type = "expense"

        debt_name = debt_category_by_id.get(int(row["debt_id"])) if row["debt_id"] is not None else None
        is_installment = (
            row["debt_id"] is not None
            and (
                debt_name == "installment"
                or category_name == "分期付款"
            )
        )
        if int(row["is_cash"] or 0) == 1:
            payment_method = "cash"
        elif row["credit_card_id"] is not None:
            payment_method = "credit_card"
        elif row["presale_id"] is not None:
            payment_method = "presale"
        elif is_installment:
            payment_method = "installment"
        else:
            payment_method = "cash"

        investment_action = None
        if invest_id is not None:
            description = row["description"] or ""
            if row["income"] not in (None, 0) and ("分红" in description or "收益" in description):
                investment_action = "dividend"
            elif row["income"] not in (None, 0):
                investment_action = "reduce_position"
            elif invest_sequence[invest_id] == 1:
                investment_action = "open_position"
            else:
                investment_action = "add_position"

        tag_list = sorted(set(tags_by_account.get(account_id, [])))
        bill_rows.append(
            (
                account_id,
                int(row["owner_id_id"]),
                row["account_date"].date(),
                category_id,
                category_name,
                bill_type,
                payment_method,
                1 if row["asset_id"] is not None else 0,
                amount,
                json.dumps(tag_list, ensure_ascii=False) if tag_list else None,
                combine_text(row["description"], row["remark"]),
                None,
                None,
                None,
                int(row["credit_card_id"]) if row["credit_card_id"] is not None else None,
                1 if is_installment else 0,
                1 if is_installment else None,
                investment_action,
                invest_id,
                invest_info[invest_id]["code"] if invest_id is not None else None,
                invest_info[invest_id]["description"] if invest_id is not None else None,
                invest_info[invest_id]["description"] if invest_id is not None else None,
                dec6(row["trade_share"]) if row["trade_share"] not in (None, 0) else None,
                int(row["asset_id"]) if row["asset_id"] is not None else None,
                "imported",
                row["account_date"],
                row["account_date"],
            )
        )

        if invest_id is not None:
            shares = Decimal(str(row["trade_share"] or 0))
            amount_decimal = Decimal(str(amount_source or 0))
            unit_price = Decimal("0")
            if shares > 0:
                unit_price = amount_decimal / shares
            elif invest_info[invest_id]["price"] is not None:
                unit_price = Decimal(str(invest_info[invest_id]["price"]))
            realized_profit = Decimal("0")
            if investment_action == "dividend":
                realized_profit = amount_decimal
            tx_rows.append(
                (
                    invest_id,
                    account_id,
                    row["account_date"].date(),
                    investment_action,
                    dec6(shares),
                    dec2(amount_decimal),
                    dec6(unit_price),
                    dec2(realized_profit),
                    combine_text(row["description"], row["remark"]),
                    row["account_date"],
                    row["account_date"],
                )
            )

    execute_many(
        dst,
        """
        INSERT INTO bills (
            id, user_id, account_date, category_id, category_name, bill_type, payment_method, is_fixed_asset,
            amount, tags, remark, transfer_group_id, transfer_target_type, transfer_target_user_id, credit_card_id,
            is_installment, installment_months, investment_action, related_investment_id, product_code, product_name,
            organization_name, share_amount, related_asset_id, special_status, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        bill_rows,
    )
    execute_many(
        dst,
        """
        INSERT INTO investment_transactions (
            investment_id, source_bill_id, transaction_date, action, shares, amount, unit_price, realized_profit, remark, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        tx_rows,
    )


def import_strategies(src, dst) -> None:
    indicator = fetch_one(src, "SELECT * FROM indicator_config ORDER BY id LIMIT 1")
    min_score = int(indicator["min_score"]) if indicator else 60
    stock_configs = fetch_all(src, "SELECT * FROM account_investstockpolicyconfig ORDER BY id")
    kdj_configs = fetch_all(src, "SELECT * FROM kdj_decision_config ORDER BY id")

    payload = []
    known_targets: set[tuple[int, str]] = set()
    for row in stock_configs:
        user_id = int(row["owner_id"] or 1)
        target_code = row["code"]
        known_targets.add((user_id, target_code))
        notes = truncate_text(
            json.dumps(
            {
                "legacy_type": "stock_policy_config",
                "ref_price": row["ref_price"],
                "dynamic_limit": row["dynamic_limit"],
                "cycle": row["cycle"],
                "cycle_save_share": row["cycle_save_share"],
                "add_steps": row["add_steps"],
                "max_t_ratio": row["max_t_ratio"],
            },
            ensure_ascii=False,
        )
        )
        payload.append(
            (
                user_id,
                "stock",
                target_code,
                f"legacy-stock-{target_code}",
                1,
                "balanced" if float(row["max_t_ratio"] or 0.3) <= 0.5 else "high",
                min_score,
                int(row["t_cool_down"] or 3),
                dec6(Decimal(str(row["stop_profit_pct"] or 0.18))),
                dec6(Decimal(str(row["add_drawdown_pct"] or 0.08))),
                notes,
            )
        )

    for row in kdj_configs:
        user_id = int(row["owner_id"] or 1)
        target_code = row["code"] or None
        if target_code and (user_id, target_code) in known_targets:
            continue
        notes = truncate_text(
            json.dumps(
            {
                "legacy_type": "kdj_decision_config",
                "day_weight": row["day_weight"],
                "week_weight": row["week_weight"],
                "month_weight": row["month_weight"],
                "build_score": row["build_score"],
                "add_score": row["add_score"],
                "reduce_score": row["reduce_score"],
            },
            ensure_ascii=False,
        )
        )
        payload.append(
            (
                user_id,
                "stock",
                target_code,
                row["name"],
                1 if row["enabled"] else 0,
                "balanced",
                int(row["build_score"] or min_score),
                3,
                dec6("0.150000"),
                dec6("0.080000"),
                notes,
            )
        )

    if indicator:
        payload.append(
            (
                1,
                "stock",
                None,
                f"legacy-global-{indicator['name']}",
                1 if indicator["active"] else 0,
                "balanced",
                int(indicator["min_score"]),
                3,
                dec6("0.180000"),
                dec6("0.080000"),
                truncate_text(
                    json.dumps({"legacy_type": "indicator_config", "raw": indicator}, ensure_ascii=False, default=str)
                ),
            )
        )

    execute_many(
        dst,
        """
        INSERT INTO strategy_configs (
            user_id, investment_type, target_code, strategy_name, enabled, risk_level, preferred_min_score,
            cooldown_days, take_profit_rate, stop_loss_rate, notes
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        payload,
    )


def import_legacy_jobs(src, dst) -> None:
    jobs = fetch_all(src, "SELECT * FROM pfm_scheduler_job ORDER BY id")
    job_rows = []
    for row in jobs:
        job_rows.append(
            (
                f"legacy_{row['code']}",
                row["name"],
                row["cron_expr"],
                1 if row["enabled"] else 0,
                100,
                1,
                int(row["max_runtime_seconds"] or 600),
                0,
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO job_configs (job_code, job_name, cron_expr, enabled, batch_size, concurrency, timeout_seconds, retry_count)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        ON DUPLICATE KEY UPDATE
            job_name = VALUES(job_name),
            cron_expr = VALUES(cron_expr),
            enabled = VALUES(enabled),
            timeout_seconds = VALUES(timeout_seconds)
        """,
        job_rows,
    )

    target_jobs = fetch_all(dst, "SELECT id, job_code FROM job_configs WHERE job_code LIKE 'legacy_%%'")
    job_id_map = {row["job_code"][7:]: int(row["id"]) for row in target_jobs}
    runs = fetch_all(src, "SELECT * FROM pfm_scheduler_run ORDER BY id")
    run_rows = []
    for row in runs:
        source_job = next((job for job in jobs if int(job["id"]) == int(row["job_id"])), None)
        if not source_job:
            continue
        target_job_id = job_id_map[source_job["code"]]
        run_rows.append(
            (
                target_job_id,
                row["status"],
                None,
                "legacy_import",
                row["started_at"],
                row["finished_at"],
                row["duration_ms"],
                row["message"],
                row["error"],
                row["started_at"],
                row["finished_at"] or row["started_at"],
            )
        )
    execute_many(
        dst,
        """
        INSERT INTO job_runs (
            job_id, status, scheduled_at, trigger_type, started_at, finished_at, duration_ms,
            message, error_message, created_at, updated_at
        ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """,
        run_rows,
    )


def import_archive_intel(src, dst) -> None:
    intel_rows: list[tuple[Any, ...]] = []

    for row in fetch_all(src, "SELECT * FROM account_financereport ORDER BY year, month"):
        intel_rows.append(
            (
                1,
                f"Legacy finance report {row['year']}-{row['month']:02d}",
                "pfm_finance_report",
                date(row["year"], row["month"], 1),
                "archived",
                "finance-report,legacy",
                f"monthly finance report {row['year']}-{row['month']:02d}",
                json.dumps(row["data"], ensure_ascii=False, default=str),
            )
        )

    for row in fetch_all(src, "SELECT * FROM account_autoinvestplan ORDER BY id"):
        intel_rows.append(
            (
                int(row["owner_id_id"]),
                f"Legacy auto invest plan #{row['id']}",
                "pfm_auto_invest",
                row["start_at"].date(),
                "archived",
                "auto-invest,legacy",
                row["description"] or "auto invest plan",
                json.dumps(row, ensure_ascii=False, default=str),
            )
        )

    for row in fetch_all(src, "SELECT * FROM account_stockaccount ORDER BY id"):
        intel_rows.append(
            (
                int(row["owner_id_id"]) if row.get("owner_id_id") is not None else int(row["owner_id"]),
                f"Legacy stock account #{row['id']}",
                "pfm_stock_account",
                row["created_at"].date(),
                "archived",
                "stock-account,legacy",
                row["name"],
                json.dumps(row, ensure_ascii=False, default=str),
            )
        )

    snapshot_rows = fetch_all(src, "SELECT * FROM account_accountsnapshot ORDER BY id")
    for row in snapshot_rows:
        intel_rows.append(
            (
                1,
                f"Legacy stock account snapshot #{row['id']}",
                "pfm_account_snapshot",
                row["date"],
                "archived",
                "stock-snapshot,legacy",
                f"asset={row['total_asset']}, position_ratio={row['position_ratio']}",
                json.dumps(row, ensure_ascii=False, default=str),
            )
        )

    unsupported_tables = [
        "account_dailynetvalue",
        "account_marketscanresult",
        "account_topstockpool",
        "account_investpolicyresult",
        "quant_nightly_run_log",
        "stock_daily_bar",
        "stock_kdj_decision",
        "stock_kdj_detail",
        "stock_technical_detail",
    ]
    for table in unsupported_tables:
        row = fetch_one(src, f"SELECT COUNT(*) AS total FROM {table}")
        intel_rows.append(
            (
                1,
                f"Legacy migration summary: {table}",
                "pfm_migration_summary",
                date.today(),
                "archived",
                "migration-summary,legacy",
                f"{table} rows retained in pfm only: {row['total']}",
                json.dumps({"table": table, "row_count": int(row["total"])}, ensure_ascii=False),
            )
        )

    execute_many(
        dst,
        """
        INSERT INTO intel_items (user_id, title, source, item_date, status, tags, summary, content)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        """,
        intel_rows,
    )


def clear_target(dst) -> None:
    execute(dst, "SET FOREIGN_KEY_CHECKS = 0")
    for table in [
        "investment_transactions",
        "stock_indicators",
        "wealth_indicators",
        "strategy_configs",
        "dashboard_snapshots",
        "job_runs",
        "bills",
        "debts",
        "presales",
        "investments",
        "assets",
        "budgets",
        "brands",
        "credit_cards",
        "balance_calibrations",
        "intel_items",
        "users",
    ]:
        execute(dst, f"DELETE FROM {table}")
    execute(dst, "DELETE FROM config_items WHERE is_builtin = 0")
    execute(dst, "DELETE FROM job_configs WHERE job_code LIKE 'legacy_%%'")
    execute(dst, "SET FOREIGN_KEY_CHECKS = 1")


def verify_source_users(src) -> None:
    rows = fetch_all(src, "SELECT username FROM auth_user ORDER BY id")
    usernames = [row["username"] for row in rows]
    required = ["admin", "孙意蔚然", "刘洁"]
    if usernames != required:
        raise RuntimeError(f"unexpected source users: {usernames}")


def main() -> None:
    src = pymysql.connect(**SOURCE_DB)
    dst = pymysql.connect(**TARGET_DB)
    try:
        verify_source_users(src)
        clear_target(dst)
        import_users(src, dst)
        category_maps = import_config_items(src, dst)
        import_credit_cards(src, dst)
        debt_category_by_id = import_debts(src, dst, category_maps)
        import_presales(src, dst, category_maps)
        import_assets(src, dst, category_maps)
        invest_info = import_investments(src, dst)
        import_budgets(src, dst, category_maps)
        import_brands(src, dst, category_maps)
        import_balance_calibrations(src, dst)
        import_bills_and_transactions(src, dst, category_maps, debt_category_by_id, invest_info)
        import_strategies(src, dst)
        import_legacy_jobs(src, dst)
        import_archive_intel(src, dst)
        dst.commit()

        summary = {
            "users": fetch_one(dst, "SELECT COUNT(*) AS total FROM users")["total"],
            "credit_cards": fetch_one(dst, "SELECT COUNT(*) AS total FROM credit_cards")["total"],
            "bills": fetch_one(dst, "SELECT COUNT(*) AS total FROM bills")["total"],
            "debts": fetch_one(dst, "SELECT COUNT(*) AS total FROM debts")["total"],
            "presales": fetch_one(dst, "SELECT COUNT(*) AS total FROM presales")["total"],
            "investments": fetch_one(dst, "SELECT COUNT(*) AS total FROM investments")["total"],
            "investment_transactions": fetch_one(dst, "SELECT COUNT(*) AS total FROM investment_transactions")["total"],
            "budgets": fetch_one(dst, "SELECT COUNT(*) AS total FROM budgets")["total"],
            "assets": fetch_one(dst, "SELECT COUNT(*) AS total FROM assets")["total"],
            "brands": fetch_one(dst, "SELECT COUNT(*) AS total FROM brands")["total"],
            "balance_calibrations": fetch_one(dst, "SELECT COUNT(*) AS total FROM balance_calibrations")["total"],
            "strategy_configs": fetch_one(dst, "SELECT COUNT(*) AS total FROM strategy_configs")["total"],
            "intel_items": fetch_one(dst, "SELECT COUNT(*) AS total FROM intel_items")["total"],
            "legacy_jobs": fetch_one(dst, "SELECT COUNT(*) AS total FROM job_configs WHERE job_code LIKE 'legacy_%%'")["total"],
        }
        print(json.dumps(summary, ensure_ascii=False, indent=2))
    except Exception:
        dst.rollback()
        raise
    finally:
        src.close()
        dst.close()


if __name__ == "__main__":
    main()
