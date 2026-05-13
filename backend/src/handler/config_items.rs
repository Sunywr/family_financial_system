use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};

use crate::{
    common::{
        response::{ok, paged},
        state::AppState,
    },
    dto::config_item::{ConfigItemListQuery, CreateConfigItemRequest, UpdateConfigItemRequest},
    error::app_error::AppError,
    service::config_item_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ConfigItemListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = config_item_service::list(&state, &query).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(config_item_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateConfigItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(config_item_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateConfigItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(config_item_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    config_item_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

pub async fn list_types() -> Result<impl IntoResponse, AppError> {
    Ok(ok(config_item_service::list_types()))
}
