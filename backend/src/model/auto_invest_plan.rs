use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AutoInvestPlan {
    pub id: u64,
    pub user_id: u64,
    pub investment_id: u64,
    pub investment_name: Option<String>,
    pub name: String,
    pub category_id: u64,
    pub category_name: String,
    pub amount: String,
    pub cycle_months: u32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub last_generated_date: Option<NaiveDate>,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
