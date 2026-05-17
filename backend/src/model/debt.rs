use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Debt {
    pub id: u64,
    pub user_id: u64,
    pub source_bill_id: Option<u64>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub repay_deadline: Option<NaiveDate>,
    pub category_id: u64,
    pub category_name: String,
    pub amount: String,
    pub period_count: u32,
    pub paid_period_count: u32,
    pub period_unit: String,
    pub period_value: u32,
    pub payment_method: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
