# FFS vs PFM Feature Alignment Matrix

- FFS endpoints discovered: 47
- PFM endpoints discovered: 58

## Capability Matrix

| category | capability | status | notes |
|---|---|---|---|
| auth | Auth: captcha | matched | Login captcha flow |
| auth | Auth: login | matched | Credential authentication |
| auth | Auth: current user | matched | Session identity endpoint |
| dashboard | Dashboard: summary/index | matched | Homepage aggregate |
| dashboard | Dashboard: cash trend | matched | Trend and obligation series |
| core | Bills CRUD | matched | Account records |
| core | Debts CRUD | matched | Debt lifecycle |
| core | Presales CRUD | matched | Presale records |
| core | Assets CRUD | matched | Assets records |
| core | Budgets CRUD | matched | Budget rules and generation |
| investments | Investments CRUD/List | matched | Investment positions |
| investments | Investments: top ranking | matched | Top candidate list |
| investments | Investment transactions | matched | FFS has dedicated transaction endpoint |
| config | Credit cards config | matched | Credit card settings |
| config | Balance calibration | matched | Balance anchor points |
| config | Brands CRUD | matched | Brand catalog |
| config | Labels/Tags | matched | Bill tags vs generic labels |
| system | Migration audit | ffs_only | FFS-only migration utility |
| system | Jobs and runs | ffs_only | FFS scheduler management endpoint set |
| system | Users CRUD | matched | PFM exposes mostly current-user info |

## Status Summary

- matched: 18
- partial: 0
- missing_in_ffs: 0
- ffs_only: 2
- missing_both: 0

## Discovered FFS Endpoints

- /api/assets
- /api/assets/{id}
- /api/auth/captcha
- /api/auth/login
- /api/auth/logout
- /api/auth/me
- /api/balance-calibrations
- /api/balance-calibrations/{id}
- /api/bill-tags
- /api/bill-tags/top
- /api/bill-tags/{id}
- /api/bills
- /api/bills/options
- /api/bills/{id}
- /api/brands
- /api/brands/{id}
- /api/budgets
- /api/budgets/generate
- /api/budgets/{id}
- /api/config/credit-cards
- /api/config/credit-cards/{id}
- /api/config/items
- /api/config/items/{id}
- /api/config/types
- /api/dashboard/cash-trend
- /api/dashboard/summary
- /api/debts
- /api/debts/{id}
- /api/health
- /api/intel
- /api/intel/{id}
- /api/investment-transactions
- /api/investments
- /api/investments/top
- /api/investments/{id}
- /api/jobs
- /api/jobs/runs
- /api/jobs/{id}
- /api/jobs/{id}/trigger
- /api/migration-audit/summary
- /api/presales
- /api/presales/{id}
- /api/strategies
- /api/strategies/{id}
- /api/users
- /api/users/options
- /api/users/{id}

## Discovered PFM Endpoints

- /api/account/account_category
- /api/account/account_creditcard
- /api/account/account_description
- /api/account/account_label_recommand
- /api/account/account_label_statistics
- /api/account/account_monthes
- /api/account/account_snapshots
- /api/account/accounts
- /api/account/api/cockpit-column-settings
- /api/account/api/indicator-config
- /api/account/apply_stock_policy_recommendation
- /api/account/asset_category
- /api/account/asset_description
- /api/account/assets
- /api/account/auto_configure_top_scan
- /api/account/auto_invest_plans
- /api/account/balance_fix
- /api/account/brands
- /api/account/budget_category
- /api/account/budget_description
- /api/account/budgets
- /api/account/category_sum
- /api/account/debt_category
- /api/account/debt_description
- /api/account/debts
- /api/account/financial_report
- /api/account/index
- /api/account/invest_category
- /api/account/invest_description
- /api/account/invest_policy
- /api/account/invest_policy_result
- /api/account/invest_risk_templates
- /api/account/invests
- /api/account/kdj_decision_config
- /api/account/kdj_decision_config_import
- /api/account/kdj_decision_config_template
- /api/account/kdj_decision_dashboard
- /api/account/label_wordcloud
- /api/account/labels
- /api/account/market_drive_dashboard
- /api/account/market_drive_evaluation
- /api/account/presale_category
- /api/account/presale_description
- /api/account/presales
- /api/account/stock_accounts
- /api/account/sub_category_sum
- /api/account/sum_per_day
- /api/account/summary
- /api/account/sync_status
- /api/account/sync_tech_cockpit
- /api/account/top_stock_pool
- /api/account/top_stock_pool/rebuild
- /api/account/top_stock_pool/rebuild_async
- /api/account/upload_brand
- /api/account/{id}/debt_repay
- /api/users/info
- /api/utils/captcha
- /api/utils/login
