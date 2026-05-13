use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct InvestmentTransaction {
    pub id: u64,
    pub investment_id: u64,
    pub source_bill_id: u64,
    pub transaction_date: NaiveDate,
    pub action: String,
    pub shares: String,
    pub amount: String,
    pub unit_price: String,
    pub realized_profit: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
