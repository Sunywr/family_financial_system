use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct DebtListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDebtRequest {
    pub user_id: u64,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub repay_deadline: Option<NaiveDate>,
    pub category_id: u64,
    pub amount: String,
    pub period_count: u32,
    pub period_unit: String,
    pub period_value: u32,
    pub payment_method: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDebtRequest {
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub repay_deadline: Option<NaiveDate>,
    pub category_id: u64,
    pub amount: String,
    pub period_count: u32,
    pub period_unit: String,
    pub period_value: u32,
    pub payment_method: String,
    pub status: String,
    pub remark: Option<String>,
}
