#!/usr/bin/env python3
"""Core alignment checker for FFS and optional PFM baseline.

This script validates core FFS capabilities and data consistency for the
current migration stage. It can optionally probe PFM endpoints to collect
baseline snapshots for manual or scripted comparison.
"""

from __future__ import annotations

import argparse
import base64
import json
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import date
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any


SVG_CODE_RE = re.compile(r">([A-Za-z0-9]{4})</text>")


@dataclass
class HttpResponse:
    status: int
    payload: dict[str, Any]


class CheckError(RuntimeError):
    """Raised when an expected validation fails."""


def http_json(
    base_url: str,
    path: str,
    method: str = "GET",
    body: dict[str, Any] | None = None,
    headers: dict[str, str] | None = None,
    params: dict[str, Any] | None = None,
    timeout: int = 20,
) -> HttpResponse:
    url = f"{base_url.rstrip('/')}{path}"
    if params:
        encoded = urllib.parse.urlencode(params, doseq=True)
        url = f"{url}?{encoded}"

    data = None
    if body is not None:
        data = json.dumps(body, ensure_ascii=False).encode("utf-8")

    request = urllib.request.Request(url=url, data=data, method=method)
    request.add_header("Accept", "application/json")
    if body is not None:
        request.add_header("Content-Type", "application/json; charset=utf-8")
    for key, value in (headers or {}).items():
        request.add_header(key, value)

    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read().decode("utf-8", errors="replace")
            payload = json.loads(raw)
            return HttpResponse(status=response.status, payload=payload)
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        raise CheckError(f"HTTP {exc.code} for {method} {url}: {raw[:300]}") from exc
    except urllib.error.URLError as exc:
        raise CheckError(f"Request failed for {method} {url}: {exc}") from exc


def extract_data_envelope(response: HttpResponse, label: str) -> dict[str, Any]:
    if "code" not in response.payload or "data" not in response.payload:
        raise CheckError(f"{label}: invalid envelope: {response.payload}")
    if response.payload["code"] != 0:
        message = response.payload.get("message", "unknown error")
        raise CheckError(f"{label}: API error code={response.payload['code']}, msg={message}")
    data = response.payload["data"]
    if not isinstance(data, dict) and not isinstance(data, list):
        raise CheckError(f"{label}: unexpected data type: {type(data).__name__}")
    return {"data": data, "message": response.payload.get("message", "")}


def decode_svg_captcha_code(image_base64: str) -> str:
    raw = image_base64
    if "," in raw:
        raw = raw.split(",", 1)[1]
    decoded = base64.b64decode(raw).decode("utf-8", errors="replace")
    match = SVG_CODE_RE.search(decoded)
    if not match:
        raise CheckError("Unable to parse captcha code from SVG image")
    return match.group(1)


def decimal_of(value: Any, label: str) -> Decimal:
    try:
        return Decimal(str(value))
    except (InvalidOperation, ValueError, TypeError) as exc:
        raise CheckError(f"Invalid decimal for {label}: {value!r}") from exc


def assert_required_fields(item: dict[str, Any], fields: list[str], label: str) -> None:
    missing = [field for field in fields if field not in item]
    if missing:
        raise CheckError(f"{label}: missing fields in first row: {missing}")


def assert_paged_payload(
    payload: Any,
    label: str,
    row_required_fields: list[str] | None = None,
) -> dict[str, Any]:
    if not isinstance(payload, dict):
        raise CheckError(f"{label}: expected object payload for paged response")

    required = ["list", "total", "page", "page_size"]
    missing = [field for field in required if field not in payload]
    if missing:
        raise CheckError(f"{label}: missing paging fields: {missing}")

    rows = payload.get("list")
    if not isinstance(rows, list):
        raise CheckError(f"{label}: list must be an array")

    if rows and row_required_fields:
        first = rows[0]
        if not isinstance(first, dict):
            raise CheckError(f"{label}: first row is not an object")
        assert_required_fields(first, row_required_fields, label)

    return {
        "total": payload.get("total"),
        "page": payload.get("page"),
        "page_size": payload.get("page_size"),
        "rows": len(rows),
    }


def login_ffs(base_url: str, username: str, password: str) -> dict[str, Any]:
    captcha_resp = http_json(base_url, "/auth/captcha")
    captcha = extract_data_envelope(captcha_resp, "ffs captcha")["data"]
    if not isinstance(captcha, dict):
        raise CheckError("ffs captcha: expected object payload")

    captcha_id = str(captcha.get("code_id", ""))
    image_base64 = str(captcha.get("image_base64", ""))
    if not captcha_id or not image_base64:
        raise CheckError("ffs captcha: missing code_id or image_base64")

    captcha_code = decode_svg_captcha_code(image_base64)
    login_resp = http_json(
        base_url,
        "/auth/login",
        method="POST",
        body={
            "username": username,
            "password": password,
            "captcha_id": captcha_id,
            "captcha_code": captcha_code,
        },
    )
    login_data = extract_data_envelope(login_resp, "ffs login")["data"]
    if not isinstance(login_data, dict) or "token" not in login_data:
        raise CheckError("ffs login: missing token")
    return login_data


def login_pfm(base_url: str, username: str, password: str) -> str:
    captcha_resp = http_json(base_url, "/api/utils/captcha")
    captcha = extract_data_envelope(captcha_resp, "pfm captcha")["data"]
    if not isinstance(captcha, dict):
        raise CheckError("pfm captcha: expected object payload")

    image = str(captcha.get("image", ""))
    captcha_id = str(captcha.get("captchaId", ""))
    if not image or not captcha_id:
        raise CheckError("pfm captcha: missing image or captchaId")

    captcha_code = decode_svg_captcha_code(image)
    login_resp = http_json(
        base_url,
        "/api/utils/login",
        method="POST",
        body={
            "username": username,
            "password": password,
            "code": captcha_code,
            "captchaId": captcha_id,
        },
    )
    login_data = extract_data_envelope(login_resp, "pfm login")["data"]
    if not isinstance(login_data, dict) or "token" not in login_data:
        raise CheckError("pfm login: missing token")
    return str(login_data["token"])


def check_ffs_core(
    ffs_base: str,
    username: str,
    password: str,
    user_id: int,
    start_date: str,
    end_date: str,
) -> dict[str, Any]:
    login_data = login_ffs(ffs_base, username, password)
    token = str(login_data["token"])
    headers = {"Authorization": f"Bearer {token}"}

    me_resp = http_json(ffs_base, "/auth/me", headers=headers)
    me_data = extract_data_envelope(me_resp, "ffs me")["data"]
    if not isinstance(me_data, dict):
        raise CheckError("ffs me: expected object payload")

    summary_resp = http_json(
        ffs_base,
        "/dashboard/summary",
        headers=headers,
        params={"user_id": user_id, "start_date": start_date, "end_date": end_date},
    )
    summary = extract_data_envelope(summary_resp, "ffs dashboard summary")["data"]
    if not isinstance(summary, dict):
        raise CheckError("ffs dashboard summary: expected object payload")

    trend_resp = http_json(
        ffs_base,
        "/dashboard/cash-trend",
        headers=headers,
        params={"user_id": user_id, "start_date": start_date, "end_date": end_date},
    )
    trend = extract_data_envelope(trend_resp, "ffs dashboard cash-trend")["data"]
    if not isinstance(trend, list):
        raise CheckError("ffs dashboard cash-trend: expected list payload")
    if not trend:
        raise CheckError("ffs dashboard cash-trend: empty list")

    audit_resp = http_json(ffs_base, "/migration-audit/summary", headers=headers)
    audit = extract_data_envelope(audit_resp, "ffs migration audit")["data"]

    bills_resp = http_json(
        ffs_base,
        "/bills",
        headers=headers,
        params={"user_id": user_id, "page": 1, "page_size": 10},
    )
    bills_data = extract_data_envelope(bills_resp, "ffs bills")["data"]

    bill_options_resp = http_json(ffs_base, "/bills/options", headers=headers)
    bill_options = extract_data_envelope(bill_options_resp, "ffs bill options")["data"]
    if not isinstance(bill_options, dict):
        raise CheckError("ffs bill options: expected object payload")
    assert_required_fields(
        bill_options,
        ["payment_methods", "normal_bill_types", "investment_bill_types", "transfer_target_types"],
        "ffs bill options",
    )

    debts_resp = http_json(
        ffs_base,
        "/debts",
        headers=headers,
        params={"user_id": user_id, "page": 1, "page_size": 10},
    )
    debts_data = extract_data_envelope(debts_resp, "ffs debts")["data"]

    investments_resp = http_json(
        ffs_base,
        "/investments",
        headers=headers,
        params={"user_id": user_id, "page": 1, "page_size": 10},
    )
    investments_data = extract_data_envelope(investments_resp, "ffs investments")["data"]

    tx_resp = http_json(
        ffs_base,
        "/investment-transactions",
        headers=headers,
        params={"page": 1, "page_size": 10},
    )
    tx_data = extract_data_envelope(tx_resp, "ffs investment transactions")["data"]

    required_summary_fields = [
        "cash_balance",
        "total_assets",
        "wealth_amount",
        "stock_amount",
        "pending_all",
        "salary_prep",
        "position_summary",
        "repay_trend",
    ]
    missing = [field for field in required_summary_fields if field not in summary]
    if missing:
        raise CheckError(f"ffs dashboard summary: missing fields: {missing}")

    summary_cash = decimal_of(summary.get("cash_balance"), "summary.cash_balance")
    last_point = trend[-1]
    if not isinstance(last_point, dict) or "cash_balance" not in last_point:
        raise CheckError("ffs dashboard cash-trend: last point missing cash_balance")
    trend_last_cash = decimal_of(last_point.get("cash_balance"), "trend[-1].cash_balance")
    if summary_cash != trend_last_cash:
        raise CheckError(
            "ffs dashboard consistency failed: "
            f"summary.cash_balance={summary_cash} != trend.last.cash_balance={trend_last_cash}"
        )

    pending_all = summary.get("pending_all", [])
    pending_count = len(pending_all) if isinstance(pending_all, list) else -1

    bills_page = assert_paged_payload(
        bills_data,
        "ffs bills",
        row_required_fields=["id", "user_id", "account_date", "amount", "bill_type", "payment_method"],
    )
    debts_page = assert_paged_payload(
        debts_data,
        "ffs debts",
        row_required_fields=["id", "user_id", "category_name", "amount", "status"],
    )
    investments_page = assert_paged_payload(
        investments_data,
        "ffs investments",
        row_required_fields=["id", "user_id", "investment_type", "name", "market_value", "status"],
    )
    transactions_page = assert_paged_payload(
        tx_data,
        "ffs investment transactions",
        row_required_fields=["id", "investment_id", "source_bill_id", "action", "amount"],
    )

    return {
        "me": {
            "id": me_data.get("id"),
            "username": me_data.get("username"),
            "display_name": me_data.get("display_name"),
            "role": me_data.get("role"),
        },
        "summary_sample": {
            "cash_balance": str(summary_cash),
            "total_assets": str(summary.get("total_assets")),
            "wealth_amount": str(summary.get("wealth_amount")),
            "stock_amount": str(summary.get("stock_amount")),
            "pending_count": pending_count,
        },
        "cash_trend": {
            "points": len(trend),
            "first_date": trend[0].get("date") if isinstance(trend[0], dict) else None,
            "last_date": last_point.get("date"),
            "last_cash_balance": str(trend_last_cash),
        },
        "list_endpoints": {
            "bills": bills_page,
            "debts": debts_page,
            "investments": investments_page,
            "investment_transactions": transactions_page,
        },
        "bill_options": {
            "payment_methods": len(bill_options.get("payment_methods", [])),
            "normal_bill_types": len(bill_options.get("normal_bill_types", [])),
            "investment_bill_types": len(bill_options.get("investment_bill_types", [])),
            "transfer_target_types": len(bill_options.get("transfer_target_types", [])),
        },
        "migration_audit_sample": audit,
    }


def collect_pfm_baseline(
    pfm_base: str,
    username: str,
    password: str,
) -> dict[str, Any]:
    token = login_pfm(pfm_base, username, password)
    headers = {"Authorization": f"Bearer {token}"}

    index_resp = http_json(pfm_base, "/api/account/index", headers=headers)
    index_data = extract_data_envelope(index_resp, "pfm index")["data"]

    summary_resp = http_json(pfm_base, "/api/account/summary", headers=headers)
    summary_data = extract_data_envelope(summary_resp, "pfm summary")["data"]

    result = {
        "index_kind": type(index_data).__name__,
        "summary_kind": type(summary_data).__name__,
        "index_keys": list(index_data.keys())[:20] if isinstance(index_data, dict) else None,
        "summary_keys": list(summary_data.keys())[:20] if isinstance(summary_data, dict) else None,
    }
    return result


def build_argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run core FFS validation and optional PFM baseline probe."
    )
    parser.add_argument("--ffs-base", default="http://127.0.0.1:8080/api", help="FFS API base URL")
    parser.add_argument("--pfm-base", default=None, help="PFM API base URL (optional)")
    parser.add_argument("--username", default="admin", help="Login username for both systems")
    parser.add_argument("--password", required=True, help="Login password for both systems")
    parser.add_argument("--user-id", type=int, default=17, help="Target user id in FFS summary APIs")
    parser.add_argument(
        "--start-date",
        default=date.today().replace(day=1).isoformat(),
        help="Summary query start date (YYYY-MM-DD)",
    )
    parser.add_argument(
        "--end-date", default=date.today().isoformat(), help="Summary query end date (YYYY-MM-DD)"
    )
    parser.add_argument(
        "--report-out",
        default="scripts/validation/out/core_check_report.json",
        help="Output JSON report path",
    )
    return parser


def main() -> int:
    parser = build_argument_parser()
    args = parser.parse_args()

    report: dict[str, Any] = {
        "status": "failed",
        "checks": {},
        "errors": [],
        "params": {
            "ffs_base": args.ffs_base,
            "pfm_base": args.pfm_base,
            "user_id": args.user_id,
            "start_date": args.start_date,
            "end_date": args.end_date,
        },
    }

    try:
        report["checks"]["ffs_core"] = check_ffs_core(
            ffs_base=args.ffs_base,
            username=args.username,
            password=args.password,
            user_id=args.user_id,
            start_date=args.start_date,
            end_date=args.end_date,
        )
        if args.pfm_base:
            report["checks"]["pfm_baseline"] = collect_pfm_baseline(
                pfm_base=args.pfm_base,
                username=args.username,
                password=args.password,
            )
        report["status"] = "passed"
    except CheckError as exc:
        report["errors"].append(str(exc))
    except Exception as exc:  # pragma: no cover - defensive fallback
        report["errors"].append(f"unexpected error: {exc!r}")

    output_path = Path(args.report_out)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")

    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
