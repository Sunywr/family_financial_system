use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct PresaleListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePresaleRequest {
    pub user_id: u64,
    pub deposit_date: NaiveDate,
    pub final_payment_date: Option<NaiveDate>,
    pub category_id: u64,
    pub deposit_amount: String,
    pub final_payment_amount: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePresaleRequest {
    pub deposit_date: NaiveDate,
    pub final_payment_date: Option<NaiveDate>,
    pub category_id: u64,
    pub deposit_amount: String,
    pub final_payment_amount: String,
    pub status: String,
    pub remark: Option<String>,
}
