use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CreditCard {
    pub id: u64,
    pub user_id: u64,
    pub name: String,
    pub billing_day: u8,
    pub repayment_day: u8,
    pub credit_limit: String,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
