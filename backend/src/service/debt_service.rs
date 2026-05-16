use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::debt::{CreateDebtRequest, DebtListQuery, UpdateDebtRequest},
    error::app_error::AppError,
    model::{config_item::ConfigItem, debt::Debt},
    repository::{config_item_repository, debt_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &DebtListQuery,
    auth_user_id: u64,
) -> Result<(Vec<Debt>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = DebtListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        status: query.status.clone(),
        keyword: query.keyword.clone(),
    };
    debt_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(state: &AppState, id: u64, auth_user_id: u64) -> Result<Debt, AppError> {
    let debt = debt_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, debt.user_id).await?;
    Ok(debt)
}

pub async fn create(
    state: &AppState,
    payload: &CreateDebtRequest,
    auth_user_id: u64,
) -> Result<Debt, AppError> {
    let category = validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = debt_repository::create(state.db()?, payload, &category.display_name, None).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateDebtRequest,
    auth_user_id: u64,
) -> Result<Debt, AppError> {
    let existing = debt_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    let category = validate_update(state, payload).await?;
    debt_repository::update(state.db()?, id, payload, &category.display_name).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = debt_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    debt_repository::soft_delete(state.db()?, id).await
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

async fn validate_create(
    state: &AppState,
    payload: &CreateDebtRequest,
) -> Result<ConfigItem, AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_category_and_values(
        &category,
        &payload.amount,
        payload.period_count,
        payload.period_value,
        &payload.period_unit,
    )?;
    Ok(category)
}

async fn validate_update(
    state: &AppState,
    payload: &UpdateDebtRequest,
) -> Result<ConfigItem, AppError> {
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_category_and_values(
        &category,
        &payload.amount,
        payload.period_count,
        payload.period_value,
        &payload.period_unit,
    )?;
    Ok(category)
}

fn validate_category_and_values(
    category: &ConfigItem,
    amount: &str,
    period_count: u32,
    period_value: u32,
    period_unit: &str,
) -> Result<(), AppError> {
    if category.config_type != "debt_category" {
        return Err(AppError::BadRequest(
            "category_id must be a debt_category".to_string(),
        ));
    }
    let decimal = Decimal::from_str(amount)
        .map_err(|_| AppError::BadRequest("amount must be a valid decimal".to_string()))?;
    if decimal <= Decimal::ZERO {
        return Err(AppError::BadRequest(
            "amount must be greater than zero".to_string(),
        ));
    }
    if period_count == 0 || period_value == 0 {
        return Err(AppError::BadRequest(
            "period_count and period_value must be positive".to_string(),
        ));
    }
    if !["day", "month", "year"].contains(&period_unit) {
        return Err(AppError::BadRequest(
            "period_unit must be day, month, or year".to_string(),
        ));
    }
    Ok(())
}
