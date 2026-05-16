#!/usr/bin/env python3
"""Generate FFS vs PFM feature alignment matrix from route definitions.

Outputs:
- JSON machine-readable matrix
- Markdown report for planning and review
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


RUST_ROUTE_RE = re.compile(r'\.route\(\s*"([^"]+)"', re.MULTILINE)
DJANGO_PATH_RE = re.compile(r'path\(\s*["\']([^"\']+)["\']')


@dataclass
class Capability:
    name: str
    category: str
    ffs_patterns: list[str]
    pfm_patterns: list[str]
    notes: str


def normalize_path(path: str) -> str:
    path = path.strip()
    if not path.startswith("/"):
        path = "/" + path
    path = re.sub(r"/+$", "", path)
    path = re.sub(r"\{[^/]+\}", "{id}", path)
    path = re.sub(r"<[^>]+>", "{id}", path)
    return path


def parse_ffs_routes(router_file: Path) -> list[str]:
    text = router_file.read_text(encoding="utf-8")
    routes = [normalize_path(m.group(1)) for m in RUST_ROUTE_RE.finditer(text)]
    return sorted(set(routes))


def parse_django_paths(file_path: Path) -> list[str]:
    text = file_path.read_text(encoding="utf-8")
    return [m.group(1).strip() for m in DJANGO_PATH_RE.finditer(text)]


def parse_pfm_routes(project_urls: Path, account_urls: Path, users_urls: Path, utils_urls: Path) -> list[str]:
    project_paths = parse_django_paths(project_urls)
    account_paths = parse_django_paths(account_urls)
    users_paths = parse_django_paths(users_urls)
    utils_paths = parse_django_paths(utils_urls)

    prefixes = {
        "api/account/": account_paths,
        "api/users/": users_paths,
        "api/utils/": utils_paths,
    }

    routes: list[str] = []
    for prefix in project_paths:
        for known_prefix, children in prefixes.items():
            if prefix.startswith(known_prefix):
                for child in children:
                    routes.append(normalize_path("/" + prefix + child))

    # Fallback: ensure direct account/users/utils routes are included even if project urls change.
    for child in account_paths:
        routes.append(normalize_path("/api/account/" + child))
    for child in users_paths:
        routes.append(normalize_path("/api/users/" + child))
    for child in utils_paths:
        routes.append(normalize_path("/api/utils/" + child))

    return sorted(set(routes))


def count_matches(patterns: list[str], endpoints: set[str]) -> tuple[int, int]:
    total = len(patterns)
    hit = sum(1 for pattern in patterns if pattern in endpoints)
    return hit, total


def classify_capability(cap: Capability, ffs_routes: set[str], pfm_routes: set[str]) -> str:
    ffs_hit, ffs_total = count_matches(cap.ffs_patterns, ffs_routes)
    pfm_hit, pfm_total = count_matches(cap.pfm_patterns, pfm_routes)

    ffs_full = ffs_total == 0 or ffs_hit == ffs_total
    pfm_full = pfm_total == 0 or pfm_hit == pfm_total
    ffs_any = ffs_hit > 0
    pfm_any = pfm_hit > 0

    if ffs_total == 0 and pfm_full:
        return "missing_both"
    if pfm_total == 0 and ffs_full:
        return "ffs_only"
    if ffs_full and pfm_full:
        return "matched"
    if ffs_any and pfm_any:
        return "partial"
    if pfm_any and not ffs_any:
        return "missing_in_ffs"
    if ffs_any and not pfm_any:
        return "ffs_only"
    return "missing_both"


def build_capabilities() -> list[Capability]:
    return [
        Capability(
            name="Auth: captcha",
            category="auth",
            ffs_patterns=["/api/auth/captcha"],
            pfm_patterns=["/api/utils/captcha"],
            notes="Login captcha flow",
        ),
        Capability(
            name="Auth: login",
            category="auth",
            ffs_patterns=["/api/auth/login"],
            pfm_patterns=["/api/utils/login"],
            notes="Credential authentication",
        ),
        Capability(
            name="Auth: current user",
            category="auth",
            ffs_patterns=["/api/auth/me"],
            pfm_patterns=["/api/users/info"],
            notes="Session identity endpoint",
        ),
        Capability(
            name="Dashboard: summary/index",
            category="dashboard",
            ffs_patterns=["/api/dashboard/summary"],
            pfm_patterns=["/api/account/index", "/api/account/summary"],
            notes="Homepage aggregate",
        ),
        Capability(
            name="Dashboard: cash trend",
            category="dashboard",
            ffs_patterns=["/api/dashboard/cash-trend"],
            pfm_patterns=["/api/account/index", "/api/account/sum_per_day"],
            notes="Trend and obligation series",
        ),
        Capability(
            name="Bills CRUD",
            category="core",
            ffs_patterns=["/api/bills", "/api/bills/{id}"],
            pfm_patterns=["/api/account/accounts"],
            notes="Account records",
        ),
        Capability(
            name="Debts CRUD",
            category="core",
            ffs_patterns=["/api/debts", "/api/debts/{id}"],
            pfm_patterns=["/api/account/debts", "/api/account/{id}/debt_repay"],
            notes="Debt lifecycle",
        ),
        Capability(
            name="Presales CRUD",
            category="core",
            ffs_patterns=["/api/presales", "/api/presales/{id}"],
            pfm_patterns=["/api/account/presales"],
            notes="Presale records",
        ),
        Capability(
            name="Assets CRUD",
            category="core",
            ffs_patterns=["/api/assets", "/api/assets/{id}"],
            pfm_patterns=["/api/account/assets"],
            notes="Assets records",
        ),
        Capability(
            name="Budgets CRUD",
            category="core",
            ffs_patterns=["/api/budgets", "/api/budgets/{id}", "/api/budgets/generate"],
            pfm_patterns=["/api/account/budgets"],
            notes="Budget rules and generation",
        ),
        Capability(
            name="Investments CRUD/List",
            category="investments",
            ffs_patterns=["/api/investments", "/api/investments/{id}"],
            pfm_patterns=["/api/account/invests"],
            notes="Investment positions",
        ),
        Capability(
            name="Investments: top ranking",
            category="investments",
            ffs_patterns=["/api/investments/top"],
            pfm_patterns=["/api/account/top_stock_pool"],
            notes="Top candidate list",
        ),
        Capability(
            name="Investment transactions",
            category="investments",
            ffs_patterns=["/api/investment-transactions"],
            pfm_patterns=["/api/account/invests"],
            notes="FFS has dedicated transaction endpoint",
        ),
        Capability(
            name="Credit cards config",
            category="config",
            ffs_patterns=["/api/config/credit-cards", "/api/config/credit-cards/{id}"],
            pfm_patterns=["/api/account/account_creditcard"],
            notes="Credit card settings",
        ),
        Capability(
            name="Balance calibration",
            category="config",
            ffs_patterns=["/api/balance-calibrations", "/api/balance-calibrations/{id}"],
            pfm_patterns=["/api/account/balance_fix"],
            notes="Balance anchor points",
        ),
        Capability(
            name="Brands CRUD",
            category="config",
            ffs_patterns=["/api/brands", "/api/brands/{id}"],
            pfm_patterns=["/api/account/brands"],
            notes="Brand catalog",
        ),
        Capability(
            name="Labels/Tags",
            category="config",
            ffs_patterns=["/api/bill-tags", "/api/bill-tags/{id}", "/api/bill-tags/top"],
            pfm_patterns=["/api/account/labels", "/api/account/label_wordcloud"],
            notes="Bill tags vs generic labels",
        ),
        Capability(
            name="Migration audit",
            category="system",
            ffs_patterns=["/api/migration-audit/summary"],
            pfm_patterns=[],
            notes="FFS-only migration utility",
        ),
        Capability(
            name="Jobs and runs",
            category="system",
            ffs_patterns=["/api/jobs", "/api/jobs/{id}", "/api/jobs/runs", "/api/jobs/{id}/trigger"],
            pfm_patterns=[],
            notes="FFS scheduler management endpoint set",
        ),
        Capability(
            name="Users CRUD",
            category="system",
            ffs_patterns=["/api/users", "/api/users/{id}", "/api/users/options"],
            pfm_patterns=["/api/users/info"],
            notes="PFM exposes mostly current-user info",
        ),
    ]


def build_markdown_report(data: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append("# FFS vs PFM Feature Alignment Matrix")
    lines.append("")
    lines.append(f"- FFS endpoints discovered: {len(data['ffs_routes'])}")
    lines.append(f"- PFM endpoints discovered: {len(data['pfm_routes'])}")
    lines.append("")

    lines.append("## Capability Matrix")
    lines.append("")
    lines.append("| category | capability | status | notes |")
    lines.append("|---|---|---|---|")
    for row in data["matrix"]:
        lines.append(
            "| {category} | {name} | {status} | {notes} |".format(
                category=row["category"],
                name=row["name"],
                status=row["status"],
                notes=row["notes"],
            )
        )

    lines.append("")
    lines.append("## Status Summary")
    lines.append("")
    summary = data["summary"]
    for key in ["matched", "partial", "missing_in_ffs", "ffs_only", "missing_both"]:
        lines.append(f"- {key}: {summary.get(key, 0)}")

    lines.append("")
    lines.append("## Discovered FFS Endpoints")
    lines.append("")
    for endpoint in data["ffs_routes"]:
        lines.append(f"- {endpoint}")

    lines.append("")
    lines.append("## Discovered PFM Endpoints")
    lines.append("")
    for endpoint in data["pfm_routes"]:
        lines.append(f"- {endpoint}")

    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate FFS vs PFM feature alignment matrix")
    parser.add_argument(
        "--ffs-router",
        default="backend/src/router/mod.rs",
        help="Path to FFS rust router file",
    )
    parser.add_argument(
        "--pfm-project-urls",
        default="../personal_financial_management/personal_financial_management/urls.py",
        help="Path to PFM project urls.py",
    )
    parser.add_argument(
        "--pfm-account-urls",
        default="../personal_financial_management/account/urls.py",
        help="Path to PFM account urls.py",
    )
    parser.add_argument(
        "--pfm-users-urls",
        default="../personal_financial_management/users/urls.py",
        help="Path to PFM users urls.py",
    )
    parser.add_argument(
        "--pfm-utils-urls",
        default="../personal_financial_management/utils/urls.py",
        help="Path to PFM utils urls.py",
    )
    parser.add_argument(
        "--json-out",
        default="scripts/validation/out/feature_alignment_matrix.json",
        help="Output JSON path",
    )
    parser.add_argument(
        "--md-out",
        default="scripts/validation/out/feature_alignment_matrix.md",
        help="Output markdown path",
    )
    args = parser.parse_args()

    ffs_routes = parse_ffs_routes(Path(args.ffs_router))
    pfm_routes = parse_pfm_routes(
        Path(args.pfm_project_urls),
        Path(args.pfm_account_urls),
        Path(args.pfm_users_urls),
        Path(args.pfm_utils_urls),
    )

    capabilities = build_capabilities()
    ffs_set = set(ffs_routes)
    pfm_set = set(pfm_routes)

    matrix: list[dict[str, Any]] = []
    summary = {
        "matched": 0,
        "partial": 0,
        "missing_in_ffs": 0,
        "ffs_only": 0,
        "missing_both": 0,
    }

    for cap in capabilities:
        status = classify_capability(cap, ffs_set, pfm_set)
        summary[status] += 1
        matrix.append(
            {
                "name": cap.name,
                "category": cap.category,
                "status": status,
                "notes": cap.notes,
                "ffs_patterns": cap.ffs_patterns,
                "pfm_patterns": cap.pfm_patterns,
            }
        )

    report = {
        "ffs_routes": ffs_routes,
        "pfm_routes": pfm_routes,
        "matrix": matrix,
        "summary": summary,
    }

    json_path = Path(args.json_out)
    md_path = Path(args.md_out)
    json_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.parent.mkdir(parents=True, exist_ok=True)

    json_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    md_path.write_text(build_markdown_report(report), encoding="utf-8")

    print(json.dumps(report["summary"], ensure_ascii=False, indent=2))
    print(f"json report: {json_path}")
    print(f"markdown report: {md_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
