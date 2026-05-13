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
    dto::intel::{CreateIntelRequest, IntelListQuery, UpdateIntelRequest},
    error::app_error::AppError,
    service::intel_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<IntelListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = intel_service::list(&state, &query).await?;
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
    Ok(ok(intel_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateIntelRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(intel_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateIntelRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(intel_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    intel_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
