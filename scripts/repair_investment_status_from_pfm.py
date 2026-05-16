import os

import pymysql


def main() -> int:
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
