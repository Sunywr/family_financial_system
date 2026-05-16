import os

import pymysql


def main() -> None:
    src = pymysql.connect(
        host=os.environ.get("PFM_DB_HOST", "127.0.0.1"),
        port=int(os.environ.get("PFM_DB_PORT", "45106")),
        user=os.environ.get("PFM_DB_USER", "pfm"),
        password=os.environ.get("PFM_DB_PASSWORD", "sywr0830"),
        database=os.environ.get("PFM_DB_NAME", "pfm"),
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )
    dst = pymysql.connect(
        host=os.environ.get("FFS_DB_HOST", "localhost"),
        port=int(os.environ.get("FFS_DB_PORT", "3306")),
        user=os.environ.get("FFS_DB_USER", "ffs"),
        password=os.environ.get("FFS_DB_PASSWORD", "sywr0830"),
        database=os.environ.get("FFS_DB_NAME", "hfs"),
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )

    try:
        with src.cursor() as src_cur:
            src_cur.execute(
                """
                SELECT id
                FROM account_account
                WHERE is_cash = 1
                  AND credit_card_id IS NOT NULL
                ORDER BY id
                """
            )
            bill_ids = [int(row["id"]) for row in src_cur.fetchall()]

        if not bill_ids:
            print("no rows to repair")
            return

        with dst.cursor() as dst_cur:
            updated = 0
            chunk_size = 500
            for start in range(0, len(bill_ids), chunk_size):
                chunk = bill_ids[start : start + chunk_size]
                placeholders = ",".join(["%s"] * len(chunk))
                sql = f"""
                    UPDATE bills
                    SET payment_method = 'cash'
                    WHERE id IN ({placeholders})
                      AND special_status = 'imported'
                      AND deleted_at IS NULL
                """
                dst_cur.execute(sql, chunk)
                updated += dst_cur.rowcount
            dst.commit()
        print(f"updated {updated} imported bills")
    finally:
        src.close()
        dst.close()


if __name__ == "__main__":
    main()
