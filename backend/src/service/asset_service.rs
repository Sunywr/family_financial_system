use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::asset::{AssetListQuery, CreateAssetRequest, UpdateAssetRequest},
    error::app_error::AppError,
    model::{asset::Asset, config_item::ConfigItem},
    repository::{asset_repository, config_item_repository, user_repository},
};

pub async fn list(state: &AppState, query: &AssetListQuery) -> Result<(Vec<Asset>, u64), AppError> {
    asset_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<Asset, AppError> {
    asset_repository::find_by_id(state.db()?, id).await
}

pub async fn create(state: &AppState, payload: &CreateAssetRequest) -> Result<Asset, AppError> {
    let category = validate_create(state, payload).await?;
    let id = asset_repository::create(state.db()?, payload, &category.display_name, None).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateAssetRequest,
) -> Result<Asset, AppError> {
    let category = validate_update(state, payload).await?;
    asset_repository::update(state.db()?, id, payload, &category.display_name).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    asset_repository::soft_delete(state.db()?, id).await
}

async fn validate_create(
    state: &AppState,
    payload: &CreateAssetRequest,
) -> Result<ConfigItem, AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_category_and_values(&category, &payload.name, &payload.amount)?;
    Ok(category)
}

async fn validate_update(
    state: &AppState,
    payload: &UpdateAssetRequest,
) -> Result<ConfigItem, AppError> {
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_category_and_values(&category, &payload.name, &payload.amount)?;
    Ok(category)
}

fn validate_category_and_values(
    category: &ConfigItem,
    name: &str,
    amount: &str,
) -> Result<(), AppError> {
    if category.config_type != "asset_category" {
        return Err(AppError::BadRequest(
            "category_id must be an asset_category".to_string(),
        ));
    }
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    let decimal = Decimal::from_str(amount)
        .map_err(|_| AppError::BadRequest("amount must be a valid decimal".to_string()))?;
    if decimal <= Decimal::ZERO {
        return Err(AppError::BadRequest(
            "amount must be greater than zero".to_string(),
        ));
    }
    Ok(())
}
