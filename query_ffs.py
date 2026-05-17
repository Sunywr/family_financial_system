import os
import pymysql
from pymysql.cursors import DictCursor

TARGET_DB = {
    "host": os.environ.get("FFS_DB_HOST", "localhost"),
    "port": int(os.environ.get("FFS_DB_PORT", "3306")),
    "user": os.environ.get("FFS_DB_USER", "ffs"),
    "password": os.environ.get("FFS_DB_PASSWORD", "sywr0830"),
    "database": os.environ.get("FFS_DB_NAME", "hfs"),
    "charset": "utf8mb4",
    "cursorclass": DictCursor,
    "autocommit": False,
}

try:
    conn = pymysql.connect(**TARGET_DB)
    with conn.cursor() as cursor:
        # Check current schema of 'bills'
        cursor.execute("DESCRIBE bills")
        bills_cols = [row['Field'] for row in cursor.fetchall()]
        
        # Debts column name might be related_debt_id in bills, but table is likely debts
        # Let's check debts table columns
        cursor.execute("DESCRIBE debts")
        debts_cols = [row['Field'] for row in cursor.fetchall()]
        print(f"Debts columns: {debts_cols}")
        
        debt_category_col = 'category' if 'category' in debts_cols else ('category_name' if 'category_name' in debts_cols else None)
        has_deleted_at = 'deleted_at' in bills_cols
        
        where_not_deleted = "b.deleted_at IS NULL" if has_deleted_at else "1=1"
        debt_filter = f"(d.id IS NULL OR d.{debt_category_col} != 'credit_card')" if debt_category_col else "1=1"

        print("--- 1 & 2: Bills (special_status='debt_cycle_auto', non-credit-card, not deleted) ---")
        query_bills = f"""
            SELECT b.user_id, COUNT(*) as count, SUM(b.amount) as total_amount
            FROM bills b
            LEFT JOIN debts d ON b.related_debt_id = d.id
            WHERE b.special_status = 'debt_cycle_auto'
              AND {where_not_deleted}
              AND {debt_filter}
            GROUP BY b.user_id WITH ROLLUP
        """
        cursor.execute(query_bills)
        results = cursor.fetchall()
        for row in results:
            uid = row['user_id'] if row['user_id'] is not None else "TOTAL"
            print(f"User ID: {uid:<10} Count: {row['count']:<5} Total Amount: {row['total_amount']}")

        print("\n--- 3: Latest pfm_homepage_summary liquid_asset (top 10 recent users) ---")
        query_summary = """
            SELECT s.user_id, s.liquid_asset, s.date
            FROM pfm_homepage_summary s
            INNER JOIN (
                SELECT user_id, MAX(date) as max_date
                FROM pfm_homepage_summary
                GROUP BY user_id
            ) latest ON s.user_id = latest.user_id AND s.date = latest.max_date
            ORDER BY s.date DESC
            LIMIT 10
        """
        try:
            cursor.execute(query_summary)
            results = cursor.fetchall()
            for row in results:
                print(f"User ID: {row['user_id']:<10} Liquid Asset: {row['liquid_asset']:<15} Date: {row['date']}")
        except Exception as e:
            print(f"Error querying pfm_homepage_summary: {e}")

except Exception as e:
    import traceback
    traceback.print_exc()
finally:
    if 'conn' in locals():
        conn.close()
