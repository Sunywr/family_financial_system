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
    dto::presale::{CreatePresaleRequest, PresaleListQuery, UpdatePresaleRequest},
    error::app_error::AppError,
    service::presale_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<PresaleListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = presale_service::list(&state, &query).await?;
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
    Ok(ok(presale_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreatePresaleRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(presale_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdatePresaleRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(presale_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    presale_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
