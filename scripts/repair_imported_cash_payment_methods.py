import pymysql


def main() -> None:
    src = pymysql.connect(
        host="127.0.0.1",
        user="pfm",
        password="sywr0830",
        database="pfm",
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )
    dst = pymysql.connect(
        host="127.0.0.1",
        user="ffs",
        password="123456",
        database="ffs",
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
