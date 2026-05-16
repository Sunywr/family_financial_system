use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPendingItem {
    pub id: Option<u64>,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub due_date: NaiveDate,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSalaryPrepSummary {
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub salary_day: u32,
    pub salary_target_date: NaiveDate,
    pub days_until_salary: i64,
    pub pending_count: usize,
    pub credit_count: u64,
    pub cycle_count: u64,
    pub credit_due_before_salary: String,
    pub cycle_due_before_salary: String,
    pub due_before_salary: String,
    pub credit_due_in_window: String,
    pub cycle_due_in_window: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPositionSummary {
    pub personal_wealth: String,
    pub personal_stock: String,
    pub total_investment: String,
    pub holding_profit: String,
    pub total_profit: String,
    pub avg_profit_rate: String,
    pub avg_annual_rate_wealth: String,
    pub family_wealth: String,
    pub family_stock: String,
    pub stock_idle_cash: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRepayTrendItem {
    pub date: NaiveDate,
    pub credit_due: String,
    pub cycle_due: String,
    pub credit_names: String,
    pub cycle_names: String,
    pub total_due: String,
    pub before_salary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRepayTrendSummary {
    pub source: String,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub salary_target_date: NaiveDate,
    pub items: Vec<DashboardRepayTrendItem>,
    pub before_salary_total: String,
    pub window_total: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummary {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub cash_balance: String,
    pub total_assets: String,
    pub outstanding_amount: String,
    pub wealth_amount: String,
    pub stock_amount: String,
    pub family_wealth_amount: String,
    pub family_stock_amount: String,
    pub stock_idle_cash: String,
    pub credit_card_outstanding_amount: String,
    pub debt_outstanding_amount: String,
    pub income: String,
    pub expense: String,
    pub net_cash_flow: String,
    pub calibration_status: String,
    pub calibration_date: Option<NaiveDate>,
    pub salary_prep: DashboardSalaryPrepSummary,
    pub position_summary: DashboardPositionSummary,
    pub repay_trend: DashboardRepayTrendSummary,
    pub credit_cards: Vec<DashboardPendingItem>,
    pub cycle_debts: Vec<DashboardPendingItem>,
    pub pending_all: Vec<DashboardPendingItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CashTrendMarkPoint {
    pub debt_id: u64,
    pub debt_name: String,
    pub debt_type: String,
    pub due_date: NaiveDate,
    pub amount: String,
    pub balance_after: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CashTrendPoint {
    pub date: NaiveDate,
    pub cash_balance: String,
    pub anchor_date: Option<NaiveDate>,
    pub mark_points: Vec<CashTrendMarkPoint>,
}
