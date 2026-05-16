# FFS 本地数据细化优化实施计划（首页优先）

## Summary
基于你确认的决策，按“首页优先”推进，先完成旧项目首页能力搬运和口径校准，再做登录/导航与各业务页可用性增强。  
已锁定的默认方案：
- 交付顺序：`首页优先`
- 验证码：`图片验证码`
- 标签范围：`仅账单标签`

目标是让 FFS 在本地数据下，首页关键指标与旧项目一致，且核心业务页具备中文可读、可检索、可追踪（显示 DB ID）的操作体验。

## 实施计划（按阶段）
1. 首页口径与趋势修复（最高优先）
- 现金余额趋势改为“与首页现金余额同口径”的日序列，确保最后一天数值恒等于首页现金余额。
- 首页 summary 增加/复用旧项目口径字段：发薪日前准备、投资持仓摘要、还款趋势、信用卡、周期债务、全部待处理（信用卡+周期债务）。
- 后端新增统一首页聚合接口（保留现有接口兼容），前端 Dashboard 改为分区卡片+模块化展示。

2. 首页内容搬运（旧项目 homepage 逻辑映射）
- 以旧项目 `homepage` 的数据口径为基准，映射到 FFS 现有表（bills/debts/investments/credit_cards/intel_items）。
- 增加“发薪日前准备”模块：应还总额、分项（信用卡/周期债务）、缺口。
- 增加“投资持仓”模块：股票与理财摘要、收益指标、持仓列表入口。
- 增加“还款趋势/待处理”模块：按日期趋势、按类型分桶、到期排序。

3. 登录页与主框架优化
- 登录页移除默认账号密码预填。
- 文案 `HFS` 全部替换为 `FFS`（登录页、侧边栏顶部、相关标题）。
- 增加图片验证码：登录前先获取验证码（code_id + image），登录时提交验证码答案校验。
- 侧边栏默认折叠；保留二级菜单结构和路由不变。

4. 业务页统一改造（账单/债务/预售/资产/投资/流水/TOP20）
- 列表统一新增：`数据库ID列`、`全文检索`。
- 状态/类型/方式统一改为中文标签（Tag）。
- 账单页：类型/方式中文标签；联动状态改为 `类型:项ID` 标签；显示账单 ID。
- 债务页：单位/状态中文标签；全文检索；显示 ID。
- 预售页：状态中文标签；新增尾款日期列；全文检索；显示 ID。
- 资产页：状态中文标签；新增创建日期列；全文检索；显示 ID。
- 投资页改名为“仪表盘”（投资子菜单）并分 `股票/理财` 页签：
  - 默认仅展示持有投资，增加“显示已出售”开关。
  - 去掉类型列，状态改中文标签，小数统一 2 位。
  - 股票显示仓位；表格上方显示 `市值 / 股票账户闲置资金 / 总收益金额`。
  - 股票类型映射：`沪A/深A/创业`；理财类型映射：`固收/基金/定投`。
  - 股票备注展示“定投周期+坝基金额”（若无数据则显示 `--`，并在后续补字段）。
- 股票流水页（投资子菜单）：动作中文标签；全文检索；显示 ID；手续费并入备注（附标记）。
- TOP20 页（投资子菜单）：类型中文标签；全文检索；显示 ID。

5. 运营配置 / 系统 / 标签能力
- 运营配置、系统模块补齐手动 `创建/修改/删除` 闭环。
- 账单标签支持：创建/配置/删除；记账输入时可实时创建并关联。

## 接口与数据结构变更（实现前先统一）
- 新增验证码接口：
  - `GET /api/auth/captcha` -> `{ code_id, image_base64 }`
  - `POST /api/auth/login` 增加字段：`captcha_code`, `captcha_id`
- 首页接口扩展（建议）：
  - `GET /api/dashboard/summary` 增加模块化字段：`salary_prep`, `position_summary`, `repay_trend`, `credit_cards`, `cycle_debts`, `pending_all`
  - `GET /api/dashboard/cash-trend` 返回与 summary 同口径的曲线，并包含校准锚点信息
- 列表接口统一新增 query：
  - `keyword`（全文检索）
  - 可选 `show_sold`（投资页）
- 标签接口（账单）：
  - `GET/POST/PUT/DELETE /api/bill-tags`
  - 账单创建/更新请求支持 `tags: string[]`

## Test Plan（验收标准）
1. 首页对齐验收（本地数据）
- 孙意蔚然账号首页四项基准值稳定一致：
  - 个人活期 `8430.31`
  - 家庭总资产 `388459.34`
  - 理财总市值 `7016.12`
  - 股票总资产 `106608.42`
- 现金余额趋势最后点必须等于首页现金余额。

2. 功能验收
- 登录页无默认预填；验证码错误时拒绝登录，正确时可登录。
- 侧边栏默认折叠，顶部标识显示 `FFS`。
- 账单/债务/预售/资产/投资/流水/TOP20 全页可全文检索并显示 DB ID。
- 投资页默认仅持有，开启开关后可见已出售。

3. 回归验收
- 后端：`cargo fmt && cargo clippy -D warnings && cargo test`
- 前端：`npm run build`
- 手工回放关键接口：dashboard、auth、bills、debts、presales、assets、investments、investment-transactions、top20。

## Assumptions
- 首页“旧项目参考口径”以 `account/view/homepage.py` 为主，而不是 `index.py` 的简版字段。
- 股票“坝基金额”若当前无明确字段，先以“备注结构化解析/空值占位”方式交付展示。
- 全文检索先做单字段拼接检索（名称/备注/代码/分类/ID），后续再按性能需要加索引优化。
- 先覆盖账单标签能力，其它模块标签暂不扩展。

## 2026-05-16 Execution Update
Completed in this round:
- Bills relation readability:
  - Backend bills list/detail now joins related user / credit card / investment / asset names.
  - Bills page relation column prefers readable names over raw ids.
- Credit card CRUD:
  - Frontend credit card page now supports create / edit / delete.
  - Backend blocks delete when bills still reference the credit card.
- Strategy page presentation:
  - Strategy config list now renders investment type and risk level as Chinese tags.
- Investment transactions readability:
  - Transactions list now shows investment name/code and source bill id.
  - Transaction keyword search now covers investment name/code and source bill id.
- Investment manual edit:
  - Added `PUT /api/investments/{id}` and frontend edit dialog.
  - Manual update recomputes derived metrics and respects ownership/admin checks.
- Homepage/dashboard:
  - Added separate dashboard sections for credit card debts / cycle debts / combined pending list.
  - Reworked summary blocks into business-oriented cards.
  - Added `system_setting/use_provident_fund_for_mortgage` toggle and wired it into cash trend projection.
- Sidebar polish:
  - Restored collapsed-menu icons and added user/logout icons.

Runtime/self-test completed:
- Backend `cargo run -- serve` started successfully.
- Health endpoint returned success with database reachable.
- Frontend `npm run dev -- --host 127.0.0.1 --port 4173` started successfully and returned HTTP 200.
- API smoke test completed through `captcha -> login -> me -> users -> dashboard/summary`.

Recommended next implementation slice:
1. Replace dashboard frontend `Record<string, unknown>` usage with typed summary sections.
2. Remove low-signal technical fields from homepage primary cards while keeping data available for debugging/tooltips.
3. If homepage continues to be the priority, consider introducing a dedicated aggregated dashboard DTO instead of the current generic JSON fragments.

## 2026-05-16 Follow-up Update
Completed after the previous execution update:
- Dashboard typing cleanup:
  - `frontend/src/api/dashboard.ts` now defines typed summary structures for `salary_prep`, `position_summary`, `repay_trend`, and pending debt rows.
  - `frontend/src/views/DashboardView.vue` was rewritten to consume those typed structures directly instead of repeatedly unpacking `Record<string, unknown>`.
  - This also removed the lingering homepage text corruption in that file and kept the current dashboard layout/behavior intact.
- Backend warning cleanup:
  - `backend/src/service/dashboard_service.rs` no longer initializes `salary_prep`, `position_summary`, and `repay_trend` with throwaway placeholder values before overwriting them.
  - The 3 existing unused-assignment warnings in dashboard service are now gone.

Verification in this follow-up:
- Frontend:
  - `npm run build` passes after the dashboard refactor.
- Backend:
  - `cargo fmt --all` passes.
  - `CARGO_TARGET_DIR=target_test cargo test` passes.
  - Plain `cargo test` can still fail if a running local backend process holds `target/debug/hfs-backend.exe`; this is an environment/process-lock issue rather than a code failure.
- Runtime:
  - `GET /api/health` returns success with database reachable.
  - Frontend dev server still returns HTTP 200 on `http://127.0.0.1:4173`.
  - Login/dashboard smoke flow was revalidated against the live backend using `captcha -> login -> me -> dashboard/summary`.

Recommended next slice now:
1. Trim low-signal dashboard fields such as `source` from the main summary cards and move them to secondary UI.
2. If homepage remains the focus, consider adding a dedicated typed backend dashboard DTO instead of generic JSON fragments.

## 2026-05-16 Homepage Presentation Follow-up
Completed in this pass:
- `frontend/src/views/DashboardView.vue`
  - cleaned up homepage copy and removed the primary-card display of technical `source` fields.
  - replaced that slot in the repayment summary with a business-facing metric: due-date count.
  - added lightweight secondary summary text for:
    - salary prep window range
    - investment annualized return / stock idle cash
    - repayment summary explanation
  - rewrote the file as a clean UTF-8 page component to eliminate the previous homepage text corruption.

Verification in this pass:
- Frontend: `npm run build` passes.
- Backend: `CARGO_TARGET_DIR=target_test cargo test` passes.
- Runtime:
  - frontend root returns HTTP 200
  - backend `/api/health` returns success
  - live smoke check through `captcha -> login -> dashboard/summary` succeeds
  - sampled summary values:
    - `cash_balance=7493.64`
    - `repay_items=10`
    - `pending_count=16`
    - `avg_annual_rate_wealth=0%`

Recommended next slice:
1. Move homepage section assembly from generic JSON blobs toward a dedicated typed backend DTO.
2. If continuing on homepage UX, consider adding compact drill-down links from summary cards into investments / bills / debts views.

## 2026-05-16 Backend DTO Follow-up
Completed in this pass:
- `backend/src/model/dashboard.rs`
  - added typed homepage sub-structures:
    - `DashboardPendingItem`
    - `DashboardSalaryPrepSummary`
    - `DashboardPositionSummary`
    - `DashboardRepayTrendItem`
    - `DashboardRepayTrendSummary`
  - `DashboardSummary` no longer exposes `serde_json::Value` / `Vec<Value>` for homepage sections.
- `backend/src/service/dashboard_service.rs`
  - replaced homepage section assembly from ad-hoc `json!` blobs with typed Rust structs.
  - retained legacy snapshot compatibility only where it still matters:
    - top metric overrides
    - fallback parsing for legacy pending lists
  - added explicit legacy parsing helpers for pending debt rows.

Verification in this pass:
- Backend:
  - `cargo fmt --all` passes.
  - `CARGO_TARGET_DIR=target_test cargo test` passes.
- Frontend:
  - `npm run build` passes unchanged against the new typed backend contract.
- Runtime compatibility check:
  - `/api/health` returns success.
  - live `dashboard/summary` still exposes compatible field paths such as:
    - `salary_prep`
    - `repay_trend.items`
    - `pending_all[].type`

Recommended next slice:
1. Add direct drill-down actions from homepage summary cards into investments, bills, and debts pages.
2. If homepage remains the focus, consider splitting the dashboard summary API into explicit sub-endpoints only if payload size or cacheability becomes a real issue.

## 2026-05-16 Homepage Drill-down Follow-up
Completed in this pass:
- `frontend/src/views/DashboardView.vue`
  - added drill-down actions on homepage summary cards:
    - salary prep -> credit-card bills / cycle debts
    - investment summary -> stock / wealth holdings
    - repayment summary -> pending bills / debt list
  - added per-row "查看明细" actions to:
    - credit card pending table
    - cycle debt pending table
    - combined pending table
- target-page query hydration:
  - `frontend/src/views/BillsView.vue` now initializes keyword/date/category/payment filters from route query.
  - `frontend/src/views/DebtsView.vue` now initializes keyword from route query.
  - `frontend/src/views/InvestmentsView.vue` now initializes `tab / keyword / show_sold / only_mine` from route query.

Verification in this pass:
- Frontend: `npm run build` passes.
- Backend: `CARGO_TARGET_DIR=target_test cargo test` passes.
- Runtime:
  - frontend root still returns HTTP 200
  - live `dashboard/summary` smoke check still succeeds
  - sampled values after this pass:
    - `credit_count=10`
    - `cycle_count=6`
    - `pending_all=16`

Recommended next slice:
1. Add reverse breadcrumbs or contextual chips on Bills / Debts / Investments when they are opened from homepage drill-downs.
2. If continuing homepage UX, consider making the repayment list rows themselves clickable cards on mobile rather than dense grid text.

## 2026-05-16 Homepage Context Hint Follow-up
Completed in this pass:
- `frontend/src/views/BillsView.vue`
  - added a contextual info banner when the page is opened from homepage drill-down.
  - banner explains the current filter source and provides a "返回首页" action.
- `frontend/src/views/DebtsView.vue`
  - rewritten as a clean UTF-8 component while preserving existing list behavior.
  - added homepage-source contextual banner and return action.
- `frontend/src/views/InvestmentsView.vue`
  - added homepage-source contextual banner and return action.
- `frontend/src/views/DashboardView.vue`
  - homepage drill-down links now pass explicit `from=dashboard` and `context=...` route query markers.

Verification in this pass:
- Frontend:
  - `npm.cmd run build` passes.
- Runtime:
  - backend `/api/health` still returns success.
- Backend:
  - no backend business logic changed in this pass; previous `CARGO_TARGET_DIR=target_test cargo test --lib --tests` result remains the relevant regression check.

Recommended next slice:
1. Improve mobile presentation of dashboard pending/repayment blocks now that they are clickable navigation surfaces.
2. Consider adding quick "clear homepage context" actions on destination pages in addition to the existing return-home shortcut.

## 2026-05-16 Investment Dashboard Consolidation
Completed in this pass:
- `frontend/src/views/InvestmentsView.vue`
  - folded "股票流水" and "TOP20" into the investment dashboard by using button-triggered drawers.
  - kept the existing holdings table, edit dialog, homepage context banner, and stock/wealth tab split.
  - drawer coverage:
    - stock transactions: keyword search + pagination
    - TOP20: type filter, admin all/mine toggle, keyword filter, pagination
- `frontend/src/layouts/AppLayout.vue`
  - removed sidebar entries for `/investment-transactions` and `/investment-top`.
  - kept "投资仪表盘" as the single investment navigation entry.
- `scripts/sync_investment_display_names.py`
  - added a repair script to backfill `investments.name` from FFS bill data first and PFM legacy history second.
  - executed the cleanup locally and finished the last few residual generic names with targeted DB updates.
- `backend/src/repository/investment_transaction_repository.rs`
  - fixed a production bug discovered during smoke testing: SQL placeholders exceeded bound parameters, causing `500` on `/api/investment-transactions`.

Verification in this pass:
- Frontend:
  - `npm run build` passes.
  - frontend dev server still returns HTTP `200` at `http://127.0.0.1:4173`.
- Backend:
  - `CARGO_TARGET_DIR=target_test cargo test --lib --tests` passes.
  - rebuilt backend binary and restarted local service.
- Runtime smoke:
  - `/api/health` returns success.
  - `captcha -> login -> me -> investments -> investments/top -> investment-transactions` succeeds after the repository fix.
  - sampled stock names now resolve to real targets such as `美的集团`, `长江电力`, `中证500ETF`, `纳指ETF`, `顺鑫农业`.

Recommended next slice:
1. Make the investment drawers navigable from inside the dashboard cards too, so homepage drill-down can optionally open TOP20 or stock transactions directly.
2. Clean up the last few wealth naming edge cases like overly short fallback text (`理财`, stripped numeric-period names) by tightening the sync script once more if needed.

## 2026-05-16 Investment Drawer Drill-down from Dashboard
Completed in this pass:
- `frontend/src/views/DashboardView.vue`
  - added "股票流水" and "TOP20 建议" action buttons to the investment summary card.
  - added `openInvestmentDrawer(drawer)` helper that navigates to investments page with `open_drawer=transactions` or `open_drawer=top20` query param.
- `frontend/src/views/InvestmentsView.vue`
  - `onMounted` now reads `open_drawer` from route query after loading data.
  - If `open_drawer=transactions`, auto-opens the stock transactions drawer.
  - If `open_drawer=top20`, auto-opens the TOP20 recommendations drawer.
  - `dashboardContextMap` extended to recognise `stock-transactions` and `top20-recommendations` context labels.

Verification in this pass:
- Frontend: `npm run build` passes.

Recommended next slice:
1. If homepage remains the focus, consider adding per-investment detail drill-down from the investment summary (e.g., clicking a holding row opens its transaction history).
2. Improve the wealth naming edge cases (overly short fallback text, stripped numeric-period names) in the sync script.
