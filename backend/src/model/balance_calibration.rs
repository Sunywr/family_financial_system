use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BalanceCalibration {
    pub id: u64,
    pub user_id: u64,
    pub calibration_date: NaiveDate,
    pub cash_balance: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
