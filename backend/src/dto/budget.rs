use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct BudgetListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub budget_month: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub user_id: u64,
    pub budget_month: NaiveDate,
    pub category_id: u64,
    pub planned_amount: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    pub budget_month: NaiveDate,
    pub category_id: u64,
    pub planned_amount: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateBudgetRequest {
    pub user_id: u64,
    pub budget_month: NaiveDate,
}
