# FFS-PFM Alignment Implementation Guide

This document is the execution baseline for the current migration stage.
It focuses on three goals:
1. Validate that FFS core capabilities match PFM core capabilities.
2. Validate that FFS data is correct against production-connected checks.
3. Redesign repeated or overly long user flows.

## 1. Execution Scope

### Included in this stage (P0/P1)
- Authentication and session flow
- Dashboard summary and cash trend
- Bills, debts, investments, assets, presales CRUD and list filtering
- Migration audit visibility
- User/config/credit-card/job admin workflows

### Deferred to dedicated tracks (P2)
- Quant/KDJ depth features
- Third-party financial provider APIs
- School district related modules

## 2. Implementation Streams

### Stream A: Functional Equivalence
- Build endpoint matrix: PFM route -> FFS route -> verification evidence.
- Run role-based flow replay:
  - Admin user full-access flow
  - Non-admin user isolation flow
- Track each gap as P0/P1/P2 with owner and ETA.

### Stream B: Data Correctness
- Build a repeatable DB + API + UI check loop.
- Use the script `scripts/validation/ffs_pfm_core_check.py` as baseline smoke checker.
- Extend with field-level diffs for monthly windows.

### Stream C: Design Rationalization
- Unify query/filter contract across pages.
- Unify status/tag dictionaries and display mapping.
- Reduce multi-step drill-down loops by keeping page context in query parameters.

## 3. First Sprint Backlog (Start Now)

### Task A1: Baseline checker bootstrap (done)
- Added script: `scripts/validation/ffs_pfm_core_check.py`
- Output: JSON report with pass/fail, key metrics, and errors.

### Task A3: Feature alignment matrix automation (done)
- Added script: `scripts/validation/generate_feature_alignment_matrix.py`
- Output:
  - `scripts/validation/out/feature_alignment_matrix.json`
  - `scripts/validation/out/feature_alignment_matrix.md`
- Purpose:
  - automatically detect route-level capability coverage
  - classify capability rows as `matched` / `partial` / `missing_in_ffs` / `ffs_only` / `missing_both`

### Task A2: Endpoint coverage expansion (in progress)
- Implemented checks for:
  - `/api/bills`
  - `/api/debts`
  - `/api/investments`
  - `/api/investment-transactions`
  - `/api/bills/options`
- Implemented assertions:
  - response envelope (`code`, `message`, `data`)
  - pagination payload (`list`, `total`, `page`, `page_size`)
  - first-row required fields for each core list endpoint

Remaining:
- Add role-specific assertions (admin vs non-admin data scope)
- Add optional keyword/date filter probes per endpoint

### Task B1: Data diff contract (next)
- Define numeric comparison tolerances.
- Define date-window policy (default: last 12 months, monthly slices).
- Add standardized diff output schema.

### Task B1 Update: Snapshot diff script implemented
- Added script: `scripts/validation/ffs_pfm_homepage_diff.py`
- Implemented:
  - FFS/PFM login and homepage fetch
  - metric mapping for core homepage indicators
  - absolute tolerance based pass/fail
  - JSON report + Markdown report output
- Current mode:
  - snapshot-level comparison (single run)
  - can be used as baseline for monthly-window extension in next step

### Task C1: Flow redesign RFC (next)
- Draft unified list-page filter model.
- Draft dashboard drill-down context contract (`from`, `context`, `keyword`, date range).

### Task C1 Update: Flow audit checklist automation (done)
- Added script: `scripts/validation/generate_flow_audit_checklist.py`
- Output:
  - `scripts/validation/out/flow_audit_checklist.json`
  - `scripts/validation/out/flow_audit_checklist.md`
- Purpose:
  - heuristically detect high-complexity pages and repeated interaction patterns
  - provide a manual redesign checklist for implementation tracking

## 4. How to Run Current Checker

Command:

```powershell
python scripts/validation/ffs_pfm_core_check.py --password <YOUR_PASSWORD>
```

Generate feature alignment matrix:

```powershell
python scripts/validation/generate_feature_alignment_matrix.py
```

Generate frontend flow redesign checklist:

```powershell
python scripts/validation/generate_flow_audit_checklist.py
```

Optional PFM baseline probe:

```powershell
python scripts/validation/ffs_pfm_core_check.py --password <YOUR_PASSWORD> --pfm-base http://127.0.0.1:8000
```

Homepage metric diff (FFS vs PFM):

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD>
```

If PFM environment uses bearer token auth, skip captcha/login:

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD> --pfm-token <PFM_BEARER_TOKEN>
```

If PFM environment uses session cookie auth, skip captcha/login:

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD> --pfm-cookie "sessionid=<...>; csrftoken=<...>"
```

If PFM captcha is not SVG and cannot be auto-decoded, provide the captcha manually:

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD> --pfm-captcha-code <CAPTCHA_CODE>
```

Bootstrap a PFM session interactively (recommended for PNG captcha mode):

```powershell
python scripts/validation/pfm_session_bootstrap.py --password <YOUR_PASSWORD>
```

This helper will:
- fetch captcha and save image to `scripts/validation/out/pfm_captcha.png`
- prompt you to input captcha code
- print token/cookie values usable by `ffs_pfm_homepage_diff.py`

Homepage metric diff with custom tolerance and user id:

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD> --ffs-user-id 17 --tolerance 0.10 --hide-mortgage
```

Monthly window diff (default last 12 full months):

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD> --mode monthly
```

Monthly window diff including current month:

```powershell
python scripts/validation/ffs_pfm_homepage_diff.py --password <YOUR_PASSWORD> --mode monthly --months 12 --include-current-month
```

Custom range and user:

```powershell
python scripts/validation/ffs_pfm_core_check.py --password <YOUR_PASSWORD> --user-id 17 --start-date 2026-01-01 --end-date 2026-05-31
```

Report output:
- Default: `scripts/validation/out/core_check_report.json`
- Diff JSON: `scripts/validation/out/homepage_diff_report.json`
- Diff Markdown: `scripts/validation/out/homepage_diff_report.md`

## 5. Acceptance Gates

- Gate 1: Script passes in stable local environment.
- Gate 2: No dashboard consistency failure (`summary.cash_balance == cash_trend.last.cash_balance`).
- Gate 3: Migration audit endpoint returns valid payload and is reviewable.
- Gate 4: Gap list for Stream A/B/C is created and tracked.

## 6. Next Immediate Engineering Actions

1. Extend checker coverage to list APIs and paging payload checks.
2. Extend homepage diff from snapshot-level to monthly windows (last 12 months).
3. Add role-scoped comparisons (admin family-view vs normal-user view).
