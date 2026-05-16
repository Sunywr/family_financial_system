use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::presale::{CreatePresaleRequest, PresaleListQuery, UpdatePresaleRequest},
    error::app_error::AppError,
    model::{config_item::ConfigItem, presale::Presale},
    repository::{config_item_repository, presale_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &PresaleListQuery,
    auth_user_id: u64,
) -> Result<(Vec<Presale>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = PresaleListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        status: query.status.clone(),
        keyword: query.keyword.clone(),
    };
    presale_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(state: &AppState, id: u64, auth_user_id: u64) -> Result<Presale, AppError> {
    let presale = presale_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, presale.user_id).await?;
    Ok(presale)
}

pub async fn create(
    state: &AppState,
    payload: &CreatePresaleRequest,
    auth_user_id: u64,
) -> Result<Presale, AppError> {
    let category = validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = presale_repository::create(state.db()?, payload, &category.display_name, None).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdatePresaleRequest,
    auth_user_id: u64,
) -> Result<Presale, AppError> {
    let existing = presale_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    let category = validate_update(state, payload).await?;
    presale_repository::update(state.db()?, id, payload, &category.display_name).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = presale_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    presale_repository::soft_delete(state.db()?, id).await
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
    payload: &CreatePresaleRequest,
) -> Result<ConfigItem, AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_category_and_values(&category, &payload.deposit_amount)?;
    if let Some(value) = &payload.final_payment_amount {
        validate_amount(value, "final_payment_amount")?;
    }
    Ok(category)
}

async fn validate_update(
    state: &AppState,
    payload: &UpdatePresaleRequest,
) -> Result<ConfigItem, AppError> {
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_category_and_values(&category, &payload.deposit_amount)?;
    validate_amount(&payload.final_payment_amount, "final_payment_amount")?;
    Ok(category)
}

fn validate_category_and_values(
    category: &ConfigItem,
    deposit_amount: &str,
) -> Result<(), AppError> {
    if category.config_type != "presale_category" {
        return Err(AppError::BadRequest(
            "category_id must be a presale_category".to_string(),
        ));
    }
    validate_amount(deposit_amount, "deposit_amount")
}

fn validate_amount(value: &str, field: &str) -> Result<(), AppError> {
    let decimal = Decimal::from_str(value)
        .map_err(|_| AppError::BadRequest(format!("{field} must be a valid decimal")))?;
    if decimal < Decimal::ZERO {
        return Err(AppError::BadRequest(format!(
            "{field} must not be negative"
        )));
    }
    Ok(())
}
