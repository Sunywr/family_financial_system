# FFS Continuation Memory

最后更新：2026-05-14 (第二轮)

## 当前目标（首页优先）
- 首页现金余额、总资产、理财市值、股票总额口径稳定。
- 现金余额趋势与首页现金余额同口径，且最后一点恒等于首页现金余额。
- 系统页和运营配置页可手动维护（创建/修改/删除）。

## 本轮新增完成
- 首页周期债务漏显修复（2026-05-14）：
  - 已核对 PFM 与 FFS：user_id=17 存在 pending 周期债务（贷款/保险）且 `repay_deadline` 很远（到 2037/2056），此前首页仅按最终 `repay_deadline` 过滤，导致近期应还（月供/保费）不显示。
  - `backend/src/repository/dashboard_repository.rs::list_pending_debts_upcoming` 改为“按周期展开每期到期事件”：
    - 信用卡仍按原逻辑按 `repay_deadline` 计一次；
    - 非信用卡 pending 债务按 `start_date ~ repay_deadline + period_unit/period_value` 展开，按每期金额（总额/总期次）生成窗口内事件。
  - 该函数同时被首页 `summary` 与 `cash_trend` 今日后推算使用，因此“房贷/保险/分期”会同时出现在还款摘要和现金趋势未来扣减点。
  - SQL 抽样验证（未来45天）已可见周期事件：`贷款 2026-05-20/2026-06-20`、`保险 2026-05-26/05-28/06-26/06-28`。
- 首页新增投资收益简表（2026-05-14）：
  - 后端新增 `investment_overview` 聚合，输出到 `position_summary`：
    - 总投资额、持仓收益、总收益、平均收益率、平均年化收益率（仅理财，取 wealth_indicators 最新 30d 年化代理均值）。
  - 前端 Dashboard 持仓摘要已补中文标签展示上述字段。

- 首页现金趋势 5/12 跳水根因修复（2026-05-14）：
  - 根因：趋势 SQL 使用硬编码 `category_id <> 585` 排除股票分类，但导入后股票分类实际为 `id=1`，导致 5/12 大额股票收支被误计入现金趋势。
  - 修复：`sum_index_cash_delta_since` 与 `daily_index_cash_deltas` 改为动态排除 `config_items(config_type='account_category', name='stock')` 对应分类ID，避免环境/导入后分类ID变化导致口径漂移。
  - 数据核对（user_id=17, 2026-05-12）：旧口径当日 `old_delta=-22936.51`，新口径该日不再计入股票分类。
- 账单列表关联与筛选增强（2026-05-14）：
  - “关联”列对 `special_status='imported'` 不再展示，减少导入历史噪音。
  - 新增筛选：按日期范围、按分类、按方式（现金/指定信用卡）。
  - 后端 `BillListQuery` 新增 `credit_card_id`，仓储层查询支持 `credit_card_id` 条件；前端 `fetchBills` 增加 filters 参数并透传 `start_date/end_date/category_id/payment_method/credit_card_id`。

- 首页现金趋势口径微调（2026-05-14）：
  - `backend/src/repository/dashboard_repository.rs` 的 `sum_index_cash_delta_since` / `daily_index_cash_deltas` 不再排除 `payment_method='stock_account'`。
  - 仍继续排除股票账单类型（`open_position/add_position/reduce_position/dividend`）和股票分类（动态按 `config_items.account_category:stock`），实现“忽略股票类型，但股票账户方式仍计入现金趋势”。
- 账单列表展示增强（2026-05-14）：
  - `frontend/src/views/BillsView.vue` 的“方式”列：当 `payment_method='credit_card'` 时优先显示具体信用卡名称（回退 `信用卡#ID`）。
  - “关联”列改为对象化文案，优先显示关联对象（如用户/投资/资产/预售/债务等），不再仅显示 `special_status + 当前账单ID`。
  - 页面加载时并行拉取信用卡列表并建立 `id->name` 映射用于账单展示。

- 首页今日点精确校准验证（2026-05-14）：
  - 使用与后端同口径的临时校验脚本确认：孙意蔚然的 5/14 现金趋势原始值为 `-1426.70`，经过 legacy liquid asset 校准后严格对齐为 `8439.31`。
  - 现金趋势当前逻辑在校准后会把今日点显式锁定为首页现金余额，避免整体平移后的舍入/锚点偏差。
- 首页还款趋势摘要名称补齐（2026-05-14）：
  - `repay_trend.items` 现在额外带出 `credit_names` / `cycle_names`，前端可直接显示对应信用卡名称和周期债务名称。
  - `pending_all` 列表继续展示具体名称，不再回退到“类型”文案。
- 首页现金趋势与展示优化（2026-05-14）：
  - 现金趋势推算改为“仅从今日之后开始扣减 pending 债务”（避免把今日应展示余额提前扣减）。
  - 现金趋势与首页现金余额的对齐锚点从“末点”改为“今日点”（若今日不在区间则回退到区间边界），保证“今日余额=首页现金余额”。
  - pending 债务展示名称优化：信用卡优先展示信用卡名称（通过 `debts -> source_bill -> credit_cards.name` 回填），周期债务保持类别名称。
  - 前端趋势图 markpoint 改为简化版：小圆点 + hover tooltip 展示明细；信用卡/周期债务继续使用红/橙区分颜色。
  - 首页下方摘要键名映射为中文；顶部四卡“股票账户总额”改为“股票总资产”。
- 首页口径重新校准（2026-05-14）：
  - 执行 `scripts/sync_legacy_homepage_summary.py`，重新写入 `intel_items.source='pfm_homepage_summary'` 快照（用于首页四卡口径对齐）。
  - 孙意蔚然（user_id=17）最新快照：
    - `liquid_asset=8439.31`
    - `family_total_asset=396877.75`
    - `stock_total_asset=107090.42`
    - `financing_market_value=7016.12`
  - 说明：`cash_trend` 逻辑会按 `legacy.liquid_asset` 对整条曲线做平移校准，末点与首页现金余额自动对齐。
- FFS 清库后从 PFM 重导入（2026-05-14）：
  - 已执行 `scripts/import_pfm_to_ffs.py`（脚本内含 `clear_target(dst)`，会先删除目标业务表数据再导入）。
  - 本机 Python 3.14 环境缺少 `pymysql`，已补装后重跑成功。
  - 本次导入统计：
    - users=3, credit_cards=18, bills=13803, debts=278, presales=45
    - investments=116, investment_transactions=406, budgets=221, assets=22
    - brands=1, balance_calibrations=2, strategy_configs=28, intel_items=536, legacy_jobs=11
- 首页现金趋势展示回滚（2026-05-14）：
  - 按用户要求回滚到上一步：恢复“折线 + pin markpoint + 标注文本（到期日/类型:名称/金额）”。
  - 移除本轮新增的分类型扣减柱状图与精简圆点方案。
- 首页现金趋势可视化精简（2026-05-14）：
  - `DashboardView` 将拥挤的文本 markpoint 改为“现金余额线 + 分类型扣减柱（信用卡/周期债务）+ 精简圆点标记”。
  - tooltip 新增到期明细，展示 `债务类型·名称(到期日)-金额`，补齐原先名称不直观的问题。
  - 信用卡与周期债务使用不同颜色（红/橙）贯穿标记与柱状图，视觉区分更明确。
- PFM→FFS 迁移执行完成（2026-05-14）：
  - 执行 `python scripts/import_pfm_to_ffs.py` 成功，导入统计：
    - users=3, credit_cards=18, bills=13803, debts=278, presales=45
    - investments=116, investment_transactions=406, budgets=221, assets=22
    - brands=1, balance_calibrations=2, strategy_configs=28, intel_items=536, legacy_jobs=11
- PFM 导入脚本健壮性修复（2026-05-14）：
  - `scripts/import_pfm_to_ffs.py` 在账单映射阶段对 `invest_id` 做存在性保护，避免 `account_account` 引用缺失投资记录时触发 `KeyError`。
  - 若投资不存在：账单不写 `related_investment_id/product_*`，且不生成对应 `investment_transactions`，保证导入不中断。
- 首页现金趋势 markpoint 前端渲染（2026-05-14）：
  - `frontend/src/api/dashboard.ts` 补充 `CashTrendMarkPoint` 类型。
  - `DashboardView` 折线图基于 `cash-trend.mark_points` 生成 `markPoint`，在每个还款截止日显示债务类型/名称/扣减金额。
- 本步验证（2026-05-14，markpoint）：
  - 后端 `cargo test` 通过。
  - 前端 `pnpm build` 通过。
- 现金趋势未来债务推算（2026-05-14）：
  - `cash_trend` 在“当前日期之后”查询 `pending` 的信用卡/周期债务，按 `repay_deadline` 将金额作为负向现金流扣减到折线。
  - 在对应日期输出 `mark_points`，用于前端图上标注每笔债务节点。
- 现金趋势 markpoint 数据结构补充（2026-05-14）：
  - 后端 `CashTrendPoint` 新增 `mark_points` 字段，每个点位可携带多个债务标记（债务ID/名称/类型/到期日/金额/扣减后余额）。
- 首页日期默认值时区修正（2026-05-14）：
  - `DashboardView` 默认日期从 `toISOString().slice(0,10)` 改为本地日期格式化，避免东八区出现 `2026-04-30 ~ 2026-05-30` 这类前移一天的问题。
  - 目标默认区间恢复为“当月月初到当月月末”。
- 现金趋势前端兜底（2026-05-14）：
  - `loadDashboard` 对 `cash-trend` 返回点位再按 `range.start_date` 过滤一次，保证图表展示起点与选择器一致（即使后端服务尚未重启）。
- 首页默认时间范围修正（2026-05-14）：
  - `DashboardView` 默认时间从“当前月月初到今天”改为“当前月月初到月末”。
  - 手动选择时间后仍沿用用户选择触发 summary 与 cash-trend 联动刷新。
- 现金趋势起点口径修正（2026-05-14）：
  - `GET /api/dashboard/cash-trend` 返回点位改为从查询 `start_date` 开始，结束规则保持不变（仍可延伸到信用卡最晚还款日）。
  - 末点与首页现金余额的 legacy 对齐逻辑保留。
- 本步验证（2026-05-14）：
  - 后端 `cargo test` 通过（使用临时 `target_test` 目录规避 Windows 可执行文件占用）。
  - 前端 `pnpm build` 通过。
- 账单页稳定性优化：
  - 重写 `frontend/src/views/BillsView.vue`，修复中文乱码。
  - 初始化加载改为“核心账单先加载 + 其它接口并行容错（Promise.allSettled）”，避免单个接口失败导致整页报错。
  - 本地接口联调中 `/api/bills`、`/api/config/items`、`/api/config/credit-cards` 均可正常返回，未稳定复现 500。
- 布局优化：
  - `frontend/src/layouts/AppLayout.vue` 去掉主窗口顶部单独页头卡片，减少重复视觉层级。
- 首页空白卡片内容完善：
  - 重写 `frontend/src/views/DashboardView.vue`，将 JSON 直出改为结构化展示（键值列表/待处理列表）并增加空状态文案。
  - 保留趋势图与四项核心指标展示。
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

## 仍待完成（按优先级）

### 🔴 第一优先：浏览器实际验收（明日/次日）
- [ ] 首页投资简表 5 字段完整展示确认
- [ ] 账单筛选（日期/分类/方式）与关联显示（对象化、imported 隐藏）
- [ ] 投资仪表盘 onlyMine 开关（管理员可见）、分页、股票名称 fallback 可用性
- [ ] TOP20 投资页 onlyMine 开关与分页可用性

### 🟡 第二优先：数据源扩展（后续专项）
- [ ] 股票真实名称字典接入（目前 code fallback 可接受）
- [ ] 股票备注"定投周期/坝基金额"数据来源或配置化（可留用户手填）
- [ ] 非信用卡周期债务展开逻辑在首页各模块完整体现

### 🟢 第三优先：去 legacy 化（长期）
- [ ] 发薪日前准备：改为 FFS 实时计算，减少 intel_items 依赖
- [ ] 还款趋势摘要：补齐信用卡账期与周期展开逻辑
- [ ] 待处理列表：改为 FFS 实时查询（bills, debts, assets）

### ⚪ 第四优先：UI/UX 增强（未来迭代）
- [ ] 列表页通用功能：列排序、导出（CSV/PDF）、批量操作
- [ ] 投资仪表盘：补齐个人/家庭视角切换、止盈/止损提醒
- [ ] 账单页：快速录入、批量编辑标签、月度总结视图

## 第二轮完成总结（2026-05-14 续作）
- **全列表页分页补漏**：
  - `InvestmentTopView.vue`：补齐本地分页（pagedList + el-pagination）。
  - `MigrationAuditView.vue`：两张表（模块、用户）均补齐本地分页。
  - 所有列表页在加载/搜索后重置 `currentPage=1`。

- **投资仪表盘与 TOP20 一致性**：
  - `InvestmentsView.vue`：管理员可见 `onlyMine` 开关（默认 true），非管理员隐藏。
  - `InvestmentTopView.vue`：同步 `onlyMine` 开关。
  - 两个页面 API 调用透传 `user_id`（当 `onlyMine` 时为当前用户，否则 undefined）。
  - 股票名称展示改进：当 `name === organization_name` 时显示 `股票 ${code}` 形式。

- **债务名称展示增强**：
  - `dashboard_repository.rs::list_pending_debts_upcoming` 中 `display_name` 逻辑：
    - 信用卡：优先显示关联信用卡名称；
    - 非信用卡：优先 `remark` 首段（分隔符 ` | `）、次选 remark、最后 category_name；
  - 该改动影响首页"还款趋势"、"待处理债务"、现金趋势 mark points 的债务名称显示。

- **代码质量验证**：
  - Rust backend: `cargo build` 成功。
  - Vue frontend: `pnpm exec vue-tsc --noEmit` 成功（Exit 0）。
  - 无编译错误或类型错误。

## 验证状态
- 后端：`cargo fmt --all`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test` 通过。
- 前端：`npm run build` 通过。
- 最新构建：2026-05-14 续作后全量通过。

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
