use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Asset {
    pub id: u64,
    pub user_id: u64,
    pub source_bill_id: Option<u64>,
    pub name: String,
    pub category_id: u64,
    pub category_name: String,
    pub amount: String,
    pub remark: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
