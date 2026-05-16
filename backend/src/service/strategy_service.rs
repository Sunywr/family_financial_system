use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::strategy::{CreateStrategyRequest, StrategyListQuery, UpdateStrategyRequest},
    error::app_error::AppError,
    model::strategy_config::StrategyConfig,
    repository::{strategy_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &StrategyListQuery,
    auth_user_id: u64,
) -> Result<(Vec<StrategyConfig>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = StrategyListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        investment_type: query.investment_type.clone(),
        enabled: query.enabled,
    };
    strategy_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(
    state: &AppState,
    id: u64,
    auth_user_id: u64,
) -> Result<StrategyConfig, AppError> {
    let strategy = strategy_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, strategy.user_id).await?;
    Ok(strategy)
}

pub async fn create(
    state: &AppState,
    payload: &CreateStrategyRequest,
    auth_user_id: u64,
) -> Result<StrategyConfig, AppError> {
    validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = strategy_repository::create(state.db()?, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateStrategyRequest,
    auth_user_id: u64,
) -> Result<StrategyConfig, AppError> {
    let existing = strategy_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    validate_update(payload)?;
    strategy_repository::update(state.db()?, id, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = strategy_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    strategy_repository::soft_delete(state.db()?, id).await
}

async fn validate_create(
    state: &AppState,
    payload: &CreateStrategyRequest,
) -> Result<(), AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    validate_fields(
        &payload.investment_type,
        payload.target_code.as_deref(),
        &payload.strategy_name,
        &payload.risk_level,
        payload.preferred_min_score,
        payload.cooldown_days,
        &payload.take_profit_rate,
        &payload.stop_loss_rate,
    )
}

fn validate_update(payload: &UpdateStrategyRequest) -> Result<(), AppError> {
    validate_fields(
        &payload.investment_type,
        payload.target_code.as_deref(),
        &payload.strategy_name,
        &payload.risk_level,
        payload.preferred_min_score,
        payload.cooldown_days,
        &payload.take_profit_rate,
        &payload.stop_loss_rate,
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_fields(
    investment_type: &str,
    target_code: Option<&str>,
    strategy_name: &str,
    risk_level: &str,
    preferred_min_score: i32,
    cooldown_days: i32,
    take_profit_rate: &str,
    stop_loss_rate: &str,
) -> Result<(), AppError> {
    if !matches!(investment_type, "stock" | "wealth") {
        return Err(AppError::BadRequest(
            "investment_type must be stock or wealth".to_string(),
        ));
    }
    if let Some(code) = target_code
        && code.trim().is_empty()
    {
        return Err(AppError::BadRequest(
            "target_code must not be empty".to_string(),
        ));
    }
    if strategy_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "strategy_name must not be empty".to_string(),
        ));
    }
    if !matches!(risk_level, "low" | "balanced" | "high") {
        return Err(AppError::BadRequest(
            "risk_level must be low balanced or high".to_string(),
        ));
    }
    if !(0..=100).contains(&preferred_min_score) {
        return Err(AppError::BadRequest(
            "preferred_min_score must be between 0 and 100".to_string(),
        ));
    }
    if !(0..=365).contains(&cooldown_days) {
        return Err(AppError::BadRequest(
            "cooldown_days must be between 0 and 365".to_string(),
        ));
    }
    let take_profit = Decimal::from_str(take_profit_rate)
        .map_err(|_| AppError::BadRequest("take_profit_rate must be decimal".to_string()))?;
    let stop_loss = Decimal::from_str(stop_loss_rate)
        .map_err(|_| AppError::BadRequest("stop_loss_rate must be decimal".to_string()))?;
    if take_profit < Decimal::ZERO || stop_loss < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "take_profit_rate and stop_loss_rate must not be negative".to_string(),
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
