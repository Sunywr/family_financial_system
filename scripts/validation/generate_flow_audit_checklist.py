#!/usr/bin/env python3
"""Generate frontend flow audit checklist for redesign planning.

Heuristic-based scanner over Vue view files to highlight:
- filter/search complexity
- pagination and drill-down context usage
- potential long/duplicated workflows
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROUTE_VIEW_RE = re.compile(r"name:\s*'([^']+)'[\s\S]*?@/views/([^']+\.vue)")


@dataclass
class ViewAudit:
    route_name: str
    view_file: str
    has_route_query: bool
    has_keyword_search: bool
    has_pagination: bool
    has_dialog: bool
    has_create_action: bool
    has_update_action: bool
    has_delete_action: bool
    has_dashboard_context: bool
    complexity_score: int


def parse_routes(router_file: Path) -> list[tuple[str, str]]:
    text = router_file.read_text(encoding="utf-8")
    matches = ROUTE_VIEW_RE.findall(text)
    return [(name, view_file) for name, view_file in matches]


def contains_any(text: str, needles: list[str]) -> bool:
    lower = text.lower()
    return any(n.lower() in lower for n in needles)


def audit_view(route_name: str, view_path: Path) -> ViewAudit:
    text = view_path.read_text(encoding="utf-8")

    has_route_query = contains_any(text, ["route.query", "useRoute(", "router.currentroute"])
    has_keyword_search = contains_any(text, ["keyword", "search", "query"])
    has_pagination = contains_any(text, ["el-pagination", "currentPage", "page_size", "pageSize"])
    has_dialog = contains_any(text, ["el-dialog", "dialogVisible", "drawer"])
    has_create_action = contains_any(text, ["create", "新增", "add"]) and contains_any(text, ["api", "post("])
    has_update_action = contains_any(text, ["update", "编辑", "put("])
    has_delete_action = contains_any(text, ["delete", "删除", "remove"]) and contains_any(text, ["delete(", "confirm"])
    has_dashboard_context = contains_any(text, ["from=dashboard", "context=", "返回首页", "dashboard"])

    score = sum(
        int(flag)
        for flag in [
            has_route_query,
            has_keyword_search,
            has_pagination,
            has_dialog,
            has_create_action,
            has_update_action,
            has_delete_action,
            has_dashboard_context,
        ]
    )

    return ViewAudit(
        route_name=route_name,
        view_file=view_path.name,
        has_route_query=has_route_query,
        has_keyword_search=has_keyword_search,
        has_pagination=has_pagination,
        has_dialog=has_dialog,
        has_create_action=has_create_action,
        has_update_action=has_update_action,
        has_delete_action=has_delete_action,
        has_dashboard_context=has_dashboard_context,
        complexity_score=score,
    )


def build_markdown(audits: list[ViewAudit]) -> str:
    lines: list[str] = []
    lines.append("# Frontend Flow Redesign Audit Checklist")
    lines.append("")
    lines.append("## View Matrix")
    lines.append("")
    lines.append(
        "| route | view | query_context | keyword | pagination | dialogs | create | update | delete | dashboard_context | score |"
    )
    lines.append("|---|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|---:|")

    for a in sorted(audits, key=lambda x: (-x.complexity_score, x.route_name)):
        lines.append(
            "| {route} | {view} | {q} | {k} | {p} | {d} | {c} | {u} | {delv} | {ctx} | {s} |".format(
                route=a.route_name,
                view=a.view_file,
                q="Y" if a.has_route_query else "N",
                k="Y" if a.has_keyword_search else "N",
                p="Y" if a.has_pagination else "N",
                d="Y" if a.has_dialog else "N",
                c="Y" if a.has_create_action else "N",
                u="Y" if a.has_update_action else "N",
                delv="Y" if a.has_delete_action else "N",
                ctx="Y" if a.has_dashboard_context else "N",
                s=a.complexity_score,
            )
        )

    candidates = [a for a in audits if a.complexity_score >= 5]
    lines.append("")
    lines.append("## Redesign Candidates (score >= 5)")
    lines.append("")
    if not candidates:
        lines.append("- None detected by heuristic rules.")
    else:
        for c in sorted(candidates, key=lambda x: (-x.complexity_score, x.route_name)):
            lines.append(f"- {c.route_name} ({c.view_file}): score={c.complexity_score}")

    lines.append("")
    lines.append("## Manual Checklist")
    lines.append("")
    lines.append("- [ ] Unify filter query model across Bills/Debts/Investments/Assets/Presales.")
    lines.append("- [ ] Ensure dashboard drill-down links always preserve and display source context.")
    lines.append("- [ ] Standardize list page toolbar order: keyword -> filters -> actions -> export.")
    lines.append("- [ ] Reduce modal nesting and repeated edit flows for high-score pages.")
    lines.append("- [ ] Ensure identical status/tag dictionary mapping across all modules.")
    lines.append("")

    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate frontend flow redesign checklist")
    parser.add_argument(
        "--router-file",
        default="frontend/src/router/index.ts",
        help="Path to frontend router file",
    )
    parser.add_argument(
        "--views-dir",
        default="frontend/src/views",
        help="Path to frontend views directory",
    )
    parser.add_argument(
        "--json-out",
        default="scripts/validation/out/flow_audit_checklist.json",
        help="Output JSON path",
    )
    parser.add_argument(
        "--md-out",
        default="scripts/validation/out/flow_audit_checklist.md",
        help="Output markdown path",
    )
    args = parser.parse_args()

    router_file = Path(args.router_file)
    views_dir = Path(args.views_dir)

    routes = parse_routes(router_file)
    audits: list[ViewAudit] = []
    for route_name, view_file in routes:
        view_path = views_dir / view_file
        if not view_path.exists():
            continue
        audits.append(audit_view(route_name, view_path))

    result: dict[str, Any] = {
        "total_routes": len(routes),
        "audited_views": len(audits),
        "rows": [
            {
                "route_name": a.route_name,
                "view_file": a.view_file,
                "has_route_query": a.has_route_query,
                "has_keyword_search": a.has_keyword_search,
                "has_pagination": a.has_pagination,
                "has_dialog": a.has_dialog,
                "has_create_action": a.has_create_action,
                "has_update_action": a.has_update_action,
                "has_delete_action": a.has_delete_action,
                "has_dashboard_context": a.has_dashboard_context,
                "complexity_score": a.complexity_score,
            }
            for a in audits
        ],
    }

    json_path = Path(args.json_out)
    md_path = Path(args.md_out)
    json_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.parent.mkdir(parents=True, exist_ok=True)

    json_path.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    md_path.write_text(build_markdown(audits), encoding="utf-8")

    print(json.dumps({
        "total_routes": result["total_routes"],
        "audited_views": result["audited_views"],
        "high_complexity": len([r for r in result["rows"] if r["complexity_score"] >= 5]),
    }, ensure_ascii=False, indent=2))
    print(f"json report: {json_path}")
    print(f"markdown report: {md_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
