import os
import re
from typing import Dict, List, Optional, Set

import pymysql


ACTION_WORDS = {
    "建仓",
    "加仓",
    "减仓",
    "赎回",
    "分红",
    "手续费",
    "买入",
    "卖出",
}
GENERIC_STOCK_ORGS = {"上海证券", "深圳证券"}
GENERIC_PROVIDER_NAMES = {
    "上海证券",
    "深圳证券",
    "广发理财",
    "上银理财",
    "信银理财",
    "招行理财",
    "交银理财",
    "兴银理财",
    "中银理财",
    "招银理财",
    "华安基金",
    "理财",
}


def connect_ffs():
    return pymysql.connect(
        host=os.environ.get("FFS_DB_HOST", "127.0.0.1"),
        port=int(os.environ.get("FFS_DB_PORT", "3306")),
        user=os.environ.get("FFS_DB_USER", "ffs"),
        password=os.environ.get("FFS_DB_PASSWORD", "sywr0830"),
        database=os.environ.get("FFS_DB_NAME", "hfs"),
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )


def connect_pfm():
    return pymysql.connect(
        host=os.environ.get("PFM_DB_HOST", "127.0.0.1"),
        port=int(os.environ.get("PFM_DB_PORT", "45106")),
        user=os.environ.get("PFM_DB_USER", "pfm"),
        password=os.environ.get("PFM_DB_PASSWORD", "sywr0830"),
        database=os.environ.get("PFM_DB_NAME", "pfm"),
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )


def normalize_text(value: Optional[str]) -> str:
    return re.sub(r"\s+", " ", (value or "").strip())


def normalize_candidate(value: str) -> str:
    cleaned = normalize_text(value)
    if "|" in cleaned:
        cleaned = normalize_text(cleaned.split("|")[-1])
    cleaned = cleaned.replace("｜", "|")
    if "|" in cleaned:
        cleaned = normalize_text(cleaned.split("|")[-1])
    if " - " in cleaned:
        cleaned = normalize_text(cleaned.rsplit(" - ", 1)[-1])
    elif "－" in cleaned:
        cleaned = normalize_text(cleaned.rsplit("－", 1)[-1])
    elif "-" in cleaned and "ETF" not in cleaned:
        cleaned = normalize_text(cleaned.rsplit("-", 1)[-1])
    cleaned = re.sub(r"^股票?(建仓|加仓|减仓)自动手续费\s*[-－]?\s*", "", cleaned)
    cleaned = re.sub(r"^(建仓|加仓|减仓|赎回|分红|买入|卖出)\s*[:：\-－|]?\s*", "", cleaned)
    cleaned = re.sub(r"\s+(赎回|建仓|加仓|减仓|买入|卖出)$", "", cleaned)
    return cleaned.strip(" |-")


def is_valid_candidate(candidate: str, organization_name: str, code: str) -> bool:
    normalized = normalize_candidate(candidate)
    if not normalized:
        return False
    if normalized in ACTION_WORDS:
        return False
    if normalized == organization_name or normalized == code:
        return False
    if normalized in GENERIC_STOCK_ORGS:
        return False
    if "手续费" in normalized:
        return False
    return len(normalized) >= 2


def extract_candidates(*parts: Optional[str]) -> List[str]:
    candidates: List[str] = []
    for part in parts:
        normalized = normalize_text(part)
        if not normalized:
            continue
        for segment in normalized.split("|"):
            segment = normalize_text(segment)
            if segment:
                candidates.append(segment)
        for splitter in (" - ", "-", "－"):
            if splitter in normalized:
                tail = normalize_text(normalized.rsplit(splitter, 1)[-1])
                if tail:
                    candidates.append(tail)
        candidates.append(normalized)
    deduped: List[str] = []
    seen: Set[str] = set()
    for candidate in candidates:
        if candidate not in seen:
            seen.add(candidate)
            deduped.append(candidate)
    return deduped


def derive_name(
    investment: Dict,
    hfs_bills: List[Dict],
    pfm_rows: List[Dict],
) -> Optional[str]:
    organization_name = normalize_text(investment["organization_name"])
    code = normalize_text(investment["code"])

    for row in hfs_bills:
        for candidate in extract_candidates(row.get("product_name"), row.get("remark")):
            if is_valid_candidate(candidate, organization_name, code):
                return normalize_candidate(candidate)

    for row in pfm_rows:
        for candidate in extract_candidates(row.get("remark"), row.get("description")):
            if is_valid_candidate(candidate, organization_name, code):
                return normalize_candidate(candidate)

    return None


def main() -> int:
    ffs = connect_ffs()
    pfm = connect_pfm()
    try:
        with ffs.cursor() as ffs_cur, pfm.cursor() as pfm_cur:
            ffs_cur.execute(
                """
                SELECT id, investment_type, code, name, organization_name
                FROM investments
                WHERE deleted_at IS NULL
                  AND (
                    name IS NULL
                    OR TRIM(name) = ''
                    OR name = organization_name
                    OR (investment_type = 'stock' AND name IN ('上海证券', '深圳证券'))
                    OR name IN ('上海证券', '深圳证券', '广发理财', '上银理财', '信银理财', '招行理财', '交银理财', '兴银理财', '中银理财', '招银理财', '华安基金', '理财')
                    OR name LIKE '%%|%%'
                    OR name REGEXP '^(建仓|加仓|减仓|赎回|分红|手续费)'
                  )
                ORDER BY id
                """
            )
            investments = ffs_cur.fetchall()

            updated = 0
            samples: List[str] = []
            for investment in investments:
                investment_id = int(investment["id"])

                ffs_cur.execute(
                    """
                    SELECT product_name, remark
                    FROM bills
                    WHERE related_investment_id = %s
                      AND deleted_at IS NULL
                    ORDER BY account_date DESC, id DESC
                    """,
                    (investment_id,),
                )
                hfs_bills = ffs_cur.fetchall()

                pfm_cur.execute(
                    """
                    SELECT description, remark
                    FROM account_account
                    WHERE invest_id = %s
                    ORDER BY account_date DESC, id DESC
                    """,
                    (investment_id,),
                )
                pfm_rows = pfm_cur.fetchall()

                new_name = derive_name(investment, hfs_bills, pfm_rows)
                if not new_name:
                    continue

                old_name = normalize_text(investment["name"])
                if old_name == new_name and old_name not in GENERIC_PROVIDER_NAMES:
                    continue
                if new_name == old_name:
                    continue

                ffs_cur.execute(
                    "UPDATE investments SET name = %s WHERE id = %s AND deleted_at IS NULL",
                    (new_name, investment_id),
                )
                if ffs_cur.rowcount:
                    updated += 1
                    if len(samples) < 12:
                        samples.append(f"{investment_id}: {old_name or '--'} -> {new_name}")

        ffs.commit()
        pfm.commit()
        print(f"updated investment rows: {updated}")
        for sample in samples:
            print(sample)
    finally:
        ffs.close()
        pfm.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
