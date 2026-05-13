use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub id: u64,
    pub user_id: u64,
    pub snapshot_date: NaiveDate,
    pub cash_balance: String,
    pub total_assets: String,
    pub outstanding_amount: String,
    pub wealth_amount: String,
    pub stock_amount: String,
    pub income: String,
    pub expense: String,
    pub net_cash_flow: String,
    pub calibration_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
