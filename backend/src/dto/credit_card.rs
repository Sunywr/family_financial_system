use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct CreditCardListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCreditCardRequest {
    pub user_id: u64,
    pub name: String,
    pub billing_day: u8,
    pub repayment_day: u8,
    pub credit_limit: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCreditCardRequest {
    pub name: String,
    pub billing_day: u8,
    pub repayment_day: u8,
    pub credit_limit: String,
    pub enabled: bool,
}
