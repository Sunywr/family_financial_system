use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Presale {
    pub id: u64,
    pub user_id: u64,
    pub source_bill_id: Option<u64>,
    pub deposit_date: NaiveDate,
    pub final_payment_date: Option<NaiveDate>,
    pub category_id: u64,
    pub category_name: String,
    pub deposit_amount: String,
    pub final_payment_amount: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
