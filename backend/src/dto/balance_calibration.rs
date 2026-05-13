use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct BalanceCalibrationListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBalanceCalibrationRequest {
    pub user_id: u64,
    pub calibration_date: NaiveDate,
    pub cash_balance: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBalanceCalibrationRequest {
    pub calibration_date: NaiveDate,
    pub cash_balance: String,
    pub remark: Option<String>,
}
