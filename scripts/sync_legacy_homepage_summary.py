import json
import os
import sys
from datetime import datetime

import pymysql


def main() -> int:
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "personal_financial_management.settings")
    legacy_repo = os.environ.get("LEGACY_PFM_REPO", r"d:\Code\personal-financial-management")
    if legacy_repo not in sys.path:
        sys.path.insert(0, legacy_repo)

    import django  # type: ignore

    django.setup()

    from django.contrib.auth.models import User  # type: ignore
    from django.test import RequestFactory  # type: ignore
    from account.view.homepage import index as legacy_homepage_index  # type: ignore

    ffs_conn = pymysql.connect(
        host="127.0.0.1",
        user="ffs",
        password="123456",
        database="ffs",
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )

    try:
        rf = RequestFactory()
        users = list(User.objects.exclude(username="admin").order_by("id"))
        now = datetime.now()

        with ffs_conn.cursor() as cur:
            for user in users:
                req = rf.get("/account/homepage")
                req.user = user
                resp = legacy_homepage_index(req)
                payload = json.loads(resp.content.decode("utf-8")).get("data", {})
                summary = payload.get("summary", {})
                content = {
                    "as_of": summary.get("as_of"),
                    "personal_total_asset": summary.get("personal_total_asset"),
                    "family_total_asset": summary.get("family_total_asset"),
                    "liquid_asset": summary.get("liquid_asset"),
                    "stock_total_asset": summary.get("stock_total_asset"),
                    "financing_market_value": summary.get("financing_market_value"),
                    "stock_available_cash": summary.get("stock_available_cash"),
                }
                cur.execute(
                    """
                    INSERT INTO intel_items (
                        user_id, title, source, item_date, tags, summary, content,
                        created_at, updated_at
                    ) VALUES (%s, %s, 'pfm_homepage_summary', %s, %s, %s, %s, %s, %s)
                    """,
                    (
                        int(user.id),
                        f"Legacy homepage summary {content.get('as_of')}",
                        content.get("as_of"),
                        "legacy,homepage,summary",
                        "legacy homepage calibrated summary",
                        json.dumps(content, ensure_ascii=False),
                        now,
                        now,
                    ),
                )
        ffs_conn.commit()
    finally:
        ffs_conn.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
