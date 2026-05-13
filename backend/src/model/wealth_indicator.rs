use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WealthIndicator {
    pub id: u64,
    pub investment_id: u64,
    pub user_id: u64,
    pub code: String,
    pub indicator_date: NaiveDate,
    pub current_nav: String,
    pub annualized_return_1d: String,
    pub annualized_return_7d: String,
    pub annualized_return_30d: String,
    pub drawdown_proxy: String,
    pub profit_rate: String,
    pub score: i32,
    pub suggestion: String,
    pub reason: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
