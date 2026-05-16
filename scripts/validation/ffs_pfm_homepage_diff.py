#!/usr/bin/env python3
"""FFS vs PFM homepage metric diff checker.

Compares core homepage metrics between:
- FFS: /api/dashboard/summary
- PFM: /api/account/index

Outputs JSON and Markdown reports for implementation tracking.
"""

from __future__ import annotations

import argparse
import base64
import http.cookiejar
import json
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any


SVG_CODE_RE = re.compile(r">([A-Za-z0-9]{4})</text>")


@dataclass
class HttpResponse:
    status: int
    payload: dict[str, Any]


class DiffError(RuntimeError):
    """Raised when the diff run fails."""


def http_json(
    base_url: str,
    path: str,
    method: str = "GET",
    body: dict[str, Any] | None = None,
    headers: dict[str, str] | None = None,
    params: dict[str, Any] | None = None,
    timeout: int = 20,
    opener: urllib.request.OpenerDirector | None = None,
) -> HttpResponse:
    url = f"{base_url.rstrip('/')}{path}"
    if params:
        url += "?" + urllib.parse.urlencode(params, doseq=True)

    data = None if body is None else json.dumps(body, ensure_ascii=False).encode("utf-8")
    req = urllib.request.Request(url=url, data=data, method=method)
    req.add_header("Accept", "application/json")
    if body is not None:
        req.add_header("Content-Type", "application/json; charset=utf-8")
    for key, value in (headers or {}).items():
        req.add_header(key, value)

    try:
        if opener is not None:
            with opener.open(req, timeout=timeout) as resp:
                raw = resp.read().decode("utf-8", errors="replace")
                return HttpResponse(status=resp.status, payload=json.loads(raw))
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8", errors="replace")
            return HttpResponse(status=resp.status, payload=json.loads(raw))
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        raise DiffError(f"HTTP {exc.code} for {method} {url}: {raw[:300]}") from exc
    except urllib.error.URLError as exc:
        raise DiffError(f"Request failed for {method} {url}: {exc}") from exc


def extract_data(response: HttpResponse, label: str) -> Any:
    payload = response.payload
    if "code" not in payload or "data" not in payload:
        raise DiffError(f"{label}: invalid envelope")
    if payload["code"] != 0:
        msg = payload.get("message", "unknown")
        raise DiffError(f"{label}: code={payload['code']}, message={msg}")
    return payload["data"]


def decode_svg_captcha_code(image_base64: str) -> str:
    raw = image_base64.split(",", 1)[1] if "," in image_base64 else image_base64
    decoded = base64.b64decode(raw).decode("utf-8", errors="replace")
    if "<svg" not in decoded.lower():
        raise DiffError("captcha image is not svg; automatic code extraction is unavailable")
    match = SVG_CODE_RE.search(decoded)
    if not match:
        raise DiffError("Unable to parse captcha code from SVG")
    return match.group(1)


def login_ffs(base_url: str, username: str, password: str) -> tuple[str, dict[str, Any]]:
    captcha_data = extract_data(http_json(base_url, "/auth/captcha"), "ffs captcha")
    if not isinstance(captcha_data, dict):
        raise DiffError("ffs captcha data is not object")

    captcha_id = str(captcha_data.get("code_id", ""))
    captcha_code = decode_svg_captcha_code(str(captcha_data.get("image_base64", "")))

    login_data = extract_data(
        http_json(
            base_url,
            "/auth/login",
            method="POST",
            body={
                "username": username,
                "password": password,
                "captcha_id": captcha_id,
                "captcha_code": captcha_code,
            },
        ),
        "ffs login",
    )
    if not isinstance(login_data, dict) or "token" not in login_data:
        raise DiffError("ffs login token missing")

    token = str(login_data["token"])
    me_data = extract_data(
        http_json(base_url, "/auth/me", headers={"Authorization": f"Bearer {token}"}),
        "ffs me",
    )
    if not isinstance(me_data, dict):
        raise DiffError("ffs me payload is not object")
    return token, me_data


def login_pfm(
    base_url: str,
    username: str,
    password: str,
    captcha_code_override: str | None = None,
) -> tuple[str, urllib.request.OpenerDirector]:
    cookie_jar = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookie_jar))

    captcha_data = extract_data(
        http_json(base_url, "/api/utils/captcha", opener=opener),
        "pfm captcha",
    )

    captcha_id = ""
    if isinstance(captcha_data, dict):
        raw_image = str(captcha_data.get("image", ""))
        captcha_id = str(captcha_data.get("captchaId", ""))
    elif isinstance(captcha_data, str):
        raw_image = captcha_data
    else:
        raise DiffError(f"pfm captcha data type unsupported: {type(captcha_data).__name__}")

    if captcha_code_override:
        captcha_code = captcha_code_override
    else:
        try:
            captcha_code = decode_svg_captcha_code(raw_image)
        except DiffError as exc:
            raise DiffError(
                "pfm captcha cannot be auto-decoded in current mode; "
                "provide --pfm-captcha-code or --pfm-token/--pfm-cookie"
            ) from exc

    login_body: dict[str, Any] = {
        "username": username,
        "password": password,
        "code": captcha_code,
    }
    if captcha_id:
        login_body["captchaId"] = captcha_id

    login_data = extract_data(
        http_json(
            base_url,
            "/api/utils/login",
            method="POST",
            body=login_body,
            opener=opener,
        ),
        "pfm login",
    )

    token = ""
    if isinstance(login_data, dict):
        token = str(login_data.get("token", ""))
    return token, opener


def get_path(data: Any, dotted_path: str) -> Any:
    current = data
    for segment in dotted_path.split("."):
        if not isinstance(current, dict):
            return None
        current = current.get(segment)
    return current


def to_decimal(value: Any) -> Decimal | None:
    if value is None:
        return None
    try:
        return Decimal(str(value))
    except (InvalidOperation, TypeError, ValueError):
        return None


def compare_metric(name: str, ffs_value: Any, pfm_value: Any, tolerance: Decimal) -> dict[str, Any]:
    ffs_num = to_decimal(ffs_value)
    pfm_num = to_decimal(pfm_value)

    if ffs_num is None or pfm_num is None:
        return {
            "name": name,
            "ffs": ffs_value,
            "pfm": pfm_value,
            "diff": None,
            "abs_diff": None,
            "within_tolerance": False,
            "reason": "non_numeric_or_missing",
        }

    diff = ffs_num - pfm_num
    abs_diff = abs(diff)
    return {
        "name": name,
        "ffs": str(ffs_num),
        "pfm": str(pfm_num),
        "diff": str(diff),
        "abs_diff": str(abs_diff),
        "within_tolerance": abs_diff <= tolerance,
        "reason": None,
    }


def month_start_end(year: int, month: int) -> tuple[str, str]:
    start = date(year, month, 1)
    if month == 12:
        end = date(year + 1, 1, 1)
    else:
        end = date(year, month + 1, 1)
    return start.isoformat(), (end.fromordinal(end.toordinal() - 1)).isoformat()


def last_n_month_windows(months: int, include_current_month: bool) -> list[tuple[str, str, str]]:
    today = date.today()
    base_month = today.month if include_current_month else (today.month - 1 or 12)
    base_year = today.year if include_current_month or today.month > 1 else today.year - 1

    windows: list[tuple[str, str, str]] = []
    year = base_year
    month = base_month
    for _ in range(months):
        start, end = month_start_end(year, month)
        label = f"{year:04d}-{month:02d}"
        windows.append((label, start, end))
        month -= 1
        if month == 0:
            month = 12
            year -= 1

    windows.reverse()
    return windows


def run_single_window(
    args: argparse.Namespace,
    tolerance: Decimal,
    ffs_headers: dict[str, str],
    ffs_user_id: int,
    pfm_headers: dict[str, str],
    pfm_opener: urllib.request.OpenerDirector | None,
    start_date: str,
    end_date: str,
    window_label: str,
) -> dict[str, Any]:
    ffs_summary = extract_data(
        http_json(
            args.ffs_base,
            "/dashboard/summary",
            headers=ffs_headers,
            params={"user_id": ffs_user_id, "start_date": start_date, "end_date": end_date},
        ),
        f"ffs summary[{window_label}]",
    )
    if not isinstance(ffs_summary, dict):
        raise DiffError(f"ffs summary payload is not object[{window_label}]")

    ffs_trend = extract_data(
        http_json(
            args.ffs_base,
            "/dashboard/cash-trend",
            headers=ffs_headers,
            params={"user_id": ffs_user_id, "start_date": start_date, "end_date": end_date},
        ),
        f"ffs cash-trend[{window_label}]",
    )
    if not isinstance(ffs_trend, list):
        raise DiffError(f"ffs cash-trend payload is not list[{window_label}]")

    pfm_index = extract_data(
        http_json(
            args.pfm_base,
            "/api/account/index",
            headers=pfm_headers,
            params={"hide_mortgage": 1 if args.hide_mortgage else 0},
            opener=pfm_opener,
        ),
        f"pfm index[{window_label}]",
    )
    if not isinstance(pfm_index, dict):
        raise DiffError(f"pfm index payload is not object[{window_label}]")

    mappings = [
        ("cash_balance", "cash_balance", "summary.liquid_asset"),
        ("total_assets", "total_assets", "summary.family_total_asset"),
        ("wealth_amount", "wealth_amount", "summary.financing_market_value"),
        ("stock_amount", "stock_amount", "summary.stock_total_asset"),
        ("stock_idle_cash", "stock_idle_cash", "summary.stock_available_cash"),
        (
            "credit_due_before_salary",
            "credit_card_outstanding_amount",
            "obligations.credit_due_before_salary",
        ),
        (
            "cycle_due_before_salary",
            "debt_outstanding_amount",
            "obligations.cycle_due_before_salary",
        ),
    ]

    metrics: list[dict[str, Any]] = []
    for metric_name, ffs_key, pfm_path in mappings:
        metrics.append(compare_metric(metric_name, ffs_summary.get(ffs_key), get_path(pfm_index, pfm_path), tolerance))

    if ffs_trend:
        last = ffs_trend[-1]
        if isinstance(last, dict):
            metrics.append(
                compare_metric(
                    "cash_trend_last_vs_pfm_liquid_asset",
                    last.get("cash_balance"),
                    get_path(pfm_index, "summary.liquid_asset"),
                    tolerance,
                )
            )

    pfm_upcoming = get_path(pfm_index, "obligations.upcoming")
    ffs_pending_all = ffs_summary.get("pending_all")

    return {
        "window": {
            "label": window_label,
            "start_date": start_date,
            "end_date": end_date,
        },
        "metrics": metrics,
        "structure": {
            "pfm_upcoming_count": len(pfm_upcoming) if isinstance(pfm_upcoming, list) else None,
            "ffs_pending_all_count": len(ffs_pending_all) if isinstance(ffs_pending_all, list) else None,
            "ffs_trend_points": len(ffs_trend),
        },
        "status": "passed" if all(item.get("within_tolerance") for item in metrics) else "failed",
    }


def build_markdown_report(report: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append("# FFS vs PFM Homepage Diff Report")
    lines.append("")
    lines.append(f"- status: {report['status']}")
    lines.append(f"- compared_at_user: {report['meta'].get('username')}")
    lines.append(f"- ffs_user_id: {report['meta'].get('ffs_user_id')}")
    lines.append(f"- tolerance: {report['meta'].get('tolerance')}")
    lines.append("")

    windows = report.get("windows")
    if isinstance(windows, list) and windows:
        lines.append("## Monthly Summary")
        lines.append("")
        lines.append("| month | status | failed_metrics |")
        lines.append("|---|:---:|---:|")
        for window in windows:
            failed_count = len([m for m in window.get("metrics", []) if not m.get("within_tolerance")])
            lines.append(
                "| {label} | {status} | {failed} |".format(
                    label=window.get("window", {}).get("label"),
                    status="PASS" if window.get("status") == "passed" else "FAIL",
                    failed=failed_count,
                )
            )

        latest = windows[-1]
        lines.append("")
        lines.append("## Latest Window Metric Comparison")
        lines.append("")
        lines.append("| metric | FFS | PFM | abs_diff | within_tolerance |")
        lines.append("|---|---:|---:|---:|:---:|")
        for item in latest.get("metrics", []):
            lines.append(
                "| {name} | {ffs} | {pfm} | {abs_diff} | {ok} |".format(
                    name=item.get("name"),
                    ffs=item.get("ffs"),
                    pfm=item.get("pfm"),
                    abs_diff=item.get("abs_diff"),
                    ok="Y" if item.get("within_tolerance") else "N",
                )
            )
    else:
        lines.append("## Metric Comparison")
        lines.append("")
        lines.append("| metric | FFS | PFM | abs_diff | within_tolerance |")
        lines.append("|---|---:|---:|---:|:---:|")
        for item in report.get("metrics", []):
            lines.append(
                "| {name} | {ffs} | {pfm} | {abs_diff} | {ok} |".format(
                    name=item.get("name"),
                    ffs=item.get("ffs"),
                    pfm=item.get("pfm"),
                    abs_diff=item.get("abs_diff"),
                    ok="Y" if item.get("within_tolerance") else "N",
                )
            )

    lines.append("")
    lines.append("## Structural Checks")
    lines.append("")
    for key, value in report.get("structure", {}).items():
        lines.append(f"- {key}: {value}")

    if report.get("errors"):
        lines.append("")
        lines.append("## Errors")
        lines.append("")
        for err in report["errors"]:
            lines.append(f"- {err}")

    lines.append("")
    return "\n".join(lines)


def run_diff(args: argparse.Namespace) -> dict[str, Any]:
    report: dict[str, Any] = {
        "status": "failed",
        "meta": {
            "ffs_base": args.ffs_base,
            "pfm_base": args.pfm_base,
            "username": args.username,
            "start_date": args.start_date,
            "end_date": args.end_date,
            "tolerance": str(args.tolerance),
            "mode": args.mode,
        },
        "metrics": [],
        "windows": [],
        "structure": {},
        "errors": [],
    }

    tolerance = Decimal(str(args.tolerance))

    ffs_token, ffs_me = login_ffs(args.ffs_base, args.username, args.password)
    ffs_user_id = args.ffs_user_id if args.ffs_user_id is not None else int(ffs_me.get("id"))
    report["meta"]["ffs_user_id"] = ffs_user_id
    ffs_headers = {"Authorization": f"Bearer {ffs_token}"}

    pfm_opener: urllib.request.OpenerDirector | None = None
    pfm_headers: dict[str, str] = {}
    if args.pfm_token:
        pfm_headers["Authorization"] = f"Bearer {args.pfm_token}"
    elif args.pfm_cookie:
        pfm_headers["Cookie"] = args.pfm_cookie
    else:
        pfm_token, pfm_opener = login_pfm(
            args.pfm_base,
            args.username,
            args.password,
            captcha_code_override=args.pfm_captcha_code,
        )
        if pfm_token:
            pfm_headers["Authorization"] = f"Bearer {pfm_token}"

    if args.mode == "monthly":
        windows = last_n_month_windows(args.months, args.include_current_month)
        window_reports: list[dict[str, Any]] = []
        for label, start_date, end_date in windows:
            window_reports.append(
                run_single_window(
                    args=args,
                    tolerance=tolerance,
                    ffs_headers=ffs_headers,
                    ffs_user_id=ffs_user_id,
                    pfm_headers=pfm_headers,
                    pfm_opener=pfm_opener,
                    start_date=start_date,
                    end_date=end_date,
                    window_label=label,
                )
            )

        report["windows"] = window_reports
        if window_reports:
            latest = window_reports[-1]
            report["metrics"] = latest.get("metrics", [])
            report["structure"] = latest.get("structure", {})
        report["status"] = "passed" if all(w.get("status") == "passed" for w in window_reports) else "failed"
    else:
        single = run_single_window(
            args=args,
            tolerance=tolerance,
            ffs_headers=ffs_headers,
            ffs_user_id=ffs_user_id,
            pfm_headers=pfm_headers,
            pfm_opener=pfm_opener,
            start_date=args.start_date,
            end_date=args.end_date,
            window_label="custom",
        )
        report["metrics"] = single.get("metrics", [])
        report["structure"] = single.get("structure", {})
        report["status"] = single.get("status", "failed")

    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Compare FFS and PFM homepage metrics.")
    parser.add_argument("--ffs-base", default="http://127.0.0.1:8080/api", help="FFS API base URL")
    parser.add_argument("--pfm-base", default="http://127.0.0.1:8000", help="PFM API base URL")
    parser.add_argument("--username", default="admin", help="Login username for both systems")
    parser.add_argument("--password", required=True, help="Login password for both systems")
    parser.add_argument(
        "--pfm-token",
        default=None,
        help="Optional PFM bearer token to skip captcha/login",
    )
    parser.add_argument(
        "--pfm-cookie",
        default=None,
        help="Optional raw PFM Cookie header value to skip captcha/login",
    )
    parser.add_argument(
        "--pfm-captcha-code",
        default=None,
        help="Manual captcha code for PFM login when auto decode is unavailable",
    )
    parser.add_argument("--ffs-user-id", type=int, default=None, help="Override FFS summary user_id")
    parser.add_argument(
        "--start-date",
        default=date.today().replace(day=1).isoformat(),
        help="FFS summary start date (YYYY-MM-DD)",
    )
    parser.add_argument(
        "--end-date",
        default=date.today().isoformat(),
        help="FFS summary end date (YYYY-MM-DD)",
    )
    parser.add_argument(
        "--tolerance",
        default="0.05",
        help="Absolute tolerance for metric comparisons",
    )
    parser.add_argument(
        "--hide-mortgage",
        action="store_true",
        help="Apply hide_mortgage=1 when requesting PFM homepage index",
    )
    parser.add_argument(
        "--mode",
        choices=["single", "monthly"],
        default="single",
        help="Run a single custom window or last N monthly windows",
    )
    parser.add_argument(
        "--months",
        type=int,
        default=12,
        help="How many months to compare in monthly mode",
    )
    parser.add_argument(
        "--include-current-month",
        action="store_true",
        help="Include the current month in monthly mode",
    )
    parser.add_argument(
        "--json-out",
        default="scripts/validation/out/homepage_diff_report.json",
        help="Path for JSON diff report",
    )
    parser.add_argument(
        "--md-out",
        default="scripts/validation/out/homepage_diff_report.md",
        help="Path for Markdown diff report",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    try:
        report = run_diff(args)
    except Exception as exc:
        report = {
            "status": "failed",
            "meta": {
                "ffs_base": args.ffs_base,
                "pfm_base": args.pfm_base,
                "username": args.username,
            },
            "metrics": [],
            "structure": {},
            "errors": [str(exc)],
        }

    json_path = Path(args.json_out)
    md_path = Path(args.md_out)
    json_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.parent.mkdir(parents=True, exist_ok=True)

    json_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    md_path.write_text(build_markdown_report(report), encoding="utf-8")

    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report.get("status") == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
