# Frontend Flow Redesign Audit Checklist

## View Matrix

| route | view | query_context | keyword | pagination | dialogs | create | update | delete | dashboard_context | score |
|---|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|---:|
| bills | BillsView.vue | Y | Y | Y | Y | Y | Y | N | Y | 7 |
| investments | InvestmentsView.vue | Y | Y | Y | Y | Y | Y | N | Y | 7 |
| config | ConfigView.vue | N | N | Y | Y | Y | Y | Y | N | 5 |
| creditCards | CreditCardsView.vue | N | N | Y | Y | Y | Y | Y | N | 5 |
| dashboard | DashboardView.vue | N | Y | N | Y | Y | Y | N | Y | 5 |
| debts | DebtsView.vue | Y | Y | Y | N | Y | N | N | Y | 5 |
| intel | IntelView.vue | N | N | Y | Y | Y | Y | N | N | 4 |
| jobs | JobsView.vue | N | N | Y | Y | Y | Y | N | N | 4 |
| strategies | StrategyConfigsView.vue | N | N | Y | Y | Y | Y | N | N | 4 |
| assets | AssetsView.vue | N | Y | Y | N | Y | N | N | N | 3 |
| investmentTop | InvestmentTopView.vue | N | Y | Y | N | Y | N | N | N | 3 |
| investmentTransactions | InvestmentTransactionsView.vue | N | Y | Y | N | Y | N | N | N | 3 |
| presales | PresalesView.vue | N | Y | Y | N | Y | N | N | N | 3 |
| balanceCalibrations | BalanceCalibrationsView.vue | N | N | Y | N | Y | N | N | N | 2 |
| brands | BrandsView.vue | N | N | Y | N | Y | N | N | N | 2 |
| budgets | BudgetsView.vue | N | N | Y | N | Y | N | N | N | 2 |
| jobRuns | JobRunsView.vue | N | N | Y | N | Y | N | N | N | 2 |
| migrationAudit | MigrationAuditView.vue | N | N | Y | N | Y | N | N | N | 2 |
| users | UsersView.vue | N | N | Y | N | Y | N | N | N | 2 |
| login | LoginView.vue | N | N | N | N | Y | N | N | N | 1 |

## Redesign Candidates (score >= 5)

- bills (BillsView.vue): score=7
- investments (InvestmentsView.vue): score=7
- config (ConfigView.vue): score=5
- creditCards (CreditCardsView.vue): score=5
- dashboard (DashboardView.vue): score=5
- debts (DebtsView.vue): score=5

## Manual Checklist

- [ ] Unify filter query model across Bills/Debts/Investments/Assets/Presales.
- [ ] Ensure dashboard drill-down links always preserve and display source context.
- [ ] Standardize list page toolbar order: keyword -> filters -> actions -> export.
- [ ] Reduce modal nesting and repeated edit flows for high-score pages.
- [ ] Ensure identical status/tag dictionary mapping across all modules.
