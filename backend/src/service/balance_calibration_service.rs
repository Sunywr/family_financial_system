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
    auth_user_id: u64,
) -> Result<(Vec<BalanceCalibration>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = BalanceCalibrationListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
    };
    balance_calibration_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(
    state: &AppState,
    id: u64,
    auth_user_id: u64,
) -> Result<BalanceCalibration, AppError> {
    let calibration = balance_calibration_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, calibration.user_id).await?;
    Ok(calibration)
}

pub async fn create(
    state: &AppState,
    payload: &CreateBalanceCalibrationRequest,
    auth_user_id: u64,
) -> Result<BalanceCalibration, AppError> {
    validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = balance_calibration_repository::create(state.db()?, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBalanceCalibrationRequest,
    auth_user_id: u64,
) -> Result<BalanceCalibration, AppError> {
    let existing = balance_calibration_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    validate_amount(&payload.cash_balance)?;
    balance_calibration_repository::update(state.db()?, id, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = balance_calibration_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
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

async fn ensure_owner_access(
    state: &AppState,
    auth_user_id: u64,
    owner_user_id: u64,
) -> Result<(), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    if auth_user.username != "admin" && owner_user_id != auth_user_id {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
