import pymysql


def main() -> int:
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
        with src.cursor() as s_cur, dst.cursor() as d_cur:
            s_cur.execute("SELECT id, status FROM account_invest ORDER BY id")
            rows = s_cur.fetchall()
            updated = 0
            for row in rows:
                new_status = "sold" if int(row["status"] or 0) == 1 else "holding"
                d_cur.execute(
                    "UPDATE investments SET status=%s WHERE id=%s AND deleted_at IS NULL",
                    (new_status, int(row["id"])),
                )
                updated += d_cur.rowcount
        dst.commit()
        src.commit()
        print(f"updated investment rows: {updated}")
    finally:
        src.close()
        dst.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
