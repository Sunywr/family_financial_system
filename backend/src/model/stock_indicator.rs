use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StockIndicator {
    pub id: u64,
    pub investment_id: u64,
    pub user_id: u64,
    pub code: String,
    pub indicator_date: NaiveDate,
    pub current_price: String,
    pub price_change_rate: String,
    pub profit_rate: String,
    pub ma_bias: String,
    pub volume_ratio: String,
    pub score: i32,
    pub suggestion: String,
    pub reason: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
