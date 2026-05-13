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
    dto::asset::{AssetListQuery, CreateAssetRequest, UpdateAssetRequest},
    error::app_error::AppError,
    service::asset_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<AssetListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = asset_service::list(&state, &query).await?;
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
    Ok(ok(asset_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateAssetRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(asset_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateAssetRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(asset_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    asset_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
