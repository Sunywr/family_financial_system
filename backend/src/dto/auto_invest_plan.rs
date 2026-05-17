use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct AutoInvestPlanListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAutoInvestPlanRequest {
    pub user_id: u64,
    pub investment_id: u64,
    pub name: String,
    pub category_id: u64,
    pub amount: String,
    pub cycle_months: u32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAutoInvestPlanRequest {
    pub name: String,
    pub category_id: u64,
    pub amount: String,
    pub cycle_months: u32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub remark: Option<String>,
}
