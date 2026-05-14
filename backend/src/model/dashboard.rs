use chrono::NaiveDate;
use serde::Serialize;

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
    pub salary_prep: serde_json::Value,
    pub position_summary: serde_json::Value,
    pub repay_trend: serde_json::Value,
    pub credit_cards: Vec<serde_json::Value>,
    pub cycle_debts: Vec<serde_json::Value>,
    pub pending_all: Vec<serde_json::Value>,
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
