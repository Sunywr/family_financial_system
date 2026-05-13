use crate::{
    common::state::AppState,
    dto::brand::{BrandListQuery, CreateBrandRequest, UpdateBrandRequest},
    error::app_error::AppError,
    model::{brand::Brand, config_item::ConfigItem},
    repository::{brand_repository, config_item_repository, user_repository},
};

pub async fn list(state: &AppState, query: &BrandListQuery) -> Result<(Vec<Brand>, u64), AppError> {
    brand_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<Brand, AppError> {
    brand_repository::find_by_id(state.db()?, id).await
}

pub async fn create(state: &AppState, payload: &CreateBrandRequest) -> Result<Brand, AppError> {
    let category = validate_create(state, payload).await?;
    let id = brand_repository::create(state.db()?, payload, &category.display_name).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBrandRequest,
) -> Result<Brand, AppError> {
    let category = validate_update(state, payload).await?;
    brand_repository::update(state.db()?, id, payload, &category.display_name).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    brand_repository::soft_delete(state.db()?, id).await
}

async fn validate_create(
    state: &AppState,
    payload: &CreateBrandRequest,
) -> Result<ConfigItem, AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_brand_fields(
        &category,
        &payload.brand_name,
        payload.score,
        &payload.board_type,
    )?;
    Ok(category)
}

async fn validate_update(
    state: &AppState,
    payload: &UpdateBrandRequest,
) -> Result<ConfigItem, AppError> {
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_brand_fields(
        &category,
        &payload.brand_name,
        payload.score,
        &payload.board_type,
    )?;
    Ok(category)
}

fn validate_brand_fields(
    category: &ConfigItem,
    brand_name: &str,
    score: i32,
    board_type: &str,
) -> Result<(), AppError> {
    if category.config_type != "brand_category" {
        return Err(AppError::BadRequest(
            "category_id must be a brand_category".to_string(),
        ));
    }
    if brand_name.trim().is_empty() {
        return Err(AppError::BadRequest("brand_name is required".to_string()));
    }
    if !(0..=100).contains(&score) {
        return Err(AppError::BadRequest(
            "score must be between 0 and 100".to_string(),
        ));
    }
    if !["red", "black", "normal"].contains(&board_type) {
        return Err(AppError::BadRequest(
            "board_type must be red, black, or normal".to_string(),
        ));
    }
    Ok(())
}
