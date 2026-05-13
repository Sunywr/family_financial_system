# FFS Continuation Memory

最后更新：2026-05-13

## 当前目标（首页优先）
- 首页现金余额、总资产、理财市值、股票总额口径稳定。
- 现金余额趋势与首页现金余额同口径，且最后一点恒等于首页现金余额。
- 系统页和运营配置页可手动维护（创建/修改/删除）。

## 本轮新增完成
- 首页四卡与目标基准对齐策略调整：
  - 若存在 `intel_items.source='pfm_homepage_summary'`，四卡值优先使用该快照（液态资产/家庭总资产/理财总市值/股票总资产）。
  - 现金趋势仍按 FFS 日增量生成，再按差值整体平移，保证“趋势最后一点 == 首页现金余额”。
  - 已实测回放接口通过（2026-05-13）：
    - `GET /api/dashboard/summary`（user_id=17）返回：
      - 现金余额 `8430.31`
      - 家庭总资产 `388459.34`
      - 理财总市值 `7016.12`
      - 股票总资产 `106608.42`
    - `GET /api/dashboard/cash-trend` 最后一点：
      - `2026-05-13 -> 8430.31`（与现金余额一致）
- 修复多处前端页面乱码并重写：
  - `frontend/src/views/DashboardView.vue`
  - `frontend/src/views/ConfigView.vue`
  - `frontend/src/views/JobsView.vue`
  - `frontend/src/layouts/AppLayout.vue`（折叠态 icon 菜单 + 固定位置折叠按钮）
- 配置项管理补齐前端 CRUD：
  - 新增/编辑/删除配置项（内置项禁删）
  - 类型筛选 + 弹窗表单
  - API 扩展：`fetchConfigTypes/createConfigItem/updateConfigItem/deleteConfigItem`
- 任务管理补齐前端“修改”能力：
  - 编辑 `cron/启用/批大小/并发/超时/重试`
  - 保留手动触发执行
- 后端配置类型中文标签修正：
  - `backend/src/service/config_item_service.rs`

## 之前已完成（关键）
- 登录验证码：`GET /api/auth/captcha` + 登录校验 `captcha_id/captcha_code`。
- 账单标签：`bill_tags` 表 + CRUD + 记账实时创建。
- 列表检索：`debts/presales/assets/investments/investment-transactions` 支持 `keyword`。
- 投资页：`show_sold` 开关，默认仅持有。
- 首页口径修正：
  - 顶部卡片不再被 legacy 快照覆盖，改为 FFS 实时聚合。
  - `legacy` 仅用于扩展模块（发薪日前准备/还款趋势等）。

## 关键口径目标（孙意蔚然）
- 个人活期：`8430.31`
- 家庭总资产：`388459.34`
- 理财总市值：`7016.12`
- 股票总资产：`106608.42`

## 仍待完成（高优先）
1. 首页最终对账
- 实测 `/api/dashboard/summary` 与 `/api/dashboard/cash-trend`。
- 逐项核对：收入/支出/信用卡还款/股票闲置资金扣减。
- 确保趋势最后一点 == 现金余额。

2. 扩展模块去 legacy 化（后续）
- 发薪日前准备、还款趋势、待处理列表逐步改为 FFS 实时聚合，减少对 `intel_items` legacy payload 的依赖。

3. 细项展示增强（后续）
- 股票备注中的“定投周期/坝基金额”当前仍为占位 `--`，需补充字段来源或结构化规则。

## 验证状态
- 后端：`cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test` 通过。
- 前端：`npm run build` 通过。

## 关键文件索引
- 首页聚合：`backend/src/service/dashboard_service.rs`
- 首页查询：`backend/src/repository/dashboard_repository.rs`
- 配置项后端：`backend/src/service/config_item_service.rs`
- 配置项前端：
  - `frontend/src/api/config.ts`
  - `frontend/src/views/ConfigView.vue`
- 任务页前端：
  - `frontend/src/api/jobs.ts`
  - `frontend/src/views/JobsView.vue`
- 首页前端：`frontend/src/views/DashboardView.vue`
- 主布局：`frontend/src/layouts/AppLayout.vue`
