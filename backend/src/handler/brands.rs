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
    dto::brand::{BrandListQuery, CreateBrandRequest, UpdateBrandRequest},
    error::app_error::AppError,
    service::brand_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<BrandListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = brand_service::list(&state, &query).await?;
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
    Ok(ok(brand_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateBrandRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(brand_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateBrandRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(brand_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    brand_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
