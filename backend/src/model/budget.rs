use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Budget {
    pub id: u64,
    pub user_id: u64,
    pub budget_month: NaiveDate,
    pub category_id: u64,
    pub category_name: String,
    pub planned_amount: String,
    pub actual_amount: String,
    pub manual_adjusted: bool,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
