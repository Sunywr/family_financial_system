use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::balance_calibration::{
        BalanceCalibrationListQuery, CreateBalanceCalibrationRequest,
        UpdateBalanceCalibrationRequest,
    },
    error::app_error::AppError,
    model::balance_calibration::BalanceCalibration,
    repository::{balance_calibration_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &BalanceCalibrationListQuery,
) -> Result<(Vec<BalanceCalibration>, u64), AppError> {
    balance_calibration_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<BalanceCalibration, AppError> {
    balance_calibration_repository::find_by_id(state.db()?, id).await
}

pub async fn create(
    state: &AppState,
    payload: &CreateBalanceCalibrationRequest,
) -> Result<BalanceCalibration, AppError> {
    validate_create(state, payload).await?;
    let id = balance_calibration_repository::create(state.db()?, payload).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBalanceCalibrationRequest,
) -> Result<BalanceCalibration, AppError> {
    validate_amount(&payload.cash_balance)?;
    balance_calibration_repository::update(state.db()?, id, payload).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    balance_calibration_repository::soft_delete(state.db()?, id).await
}

async fn validate_create(
    state: &AppState,
    payload: &CreateBalanceCalibrationRequest,
) -> Result<(), AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    validate_amount(&payload.cash_balance)
}

fn validate_amount(value: &str) -> Result<(), AppError> {
    let decimal = Decimal::from_str(value)
        .map_err(|_| AppError::BadRequest("cash_balance must be a valid decimal".to_string()))?;
    if decimal < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "cash_balance must not be negative".to_string(),
        ));
    }
    Ok(())
}
