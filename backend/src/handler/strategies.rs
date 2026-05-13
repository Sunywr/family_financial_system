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
    dto::strategy::{CreateStrategyRequest, StrategyListQuery, UpdateStrategyRequest},
    error::app_error::AppError,
    service::strategy_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<StrategyListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = strategy_service::list(&state, &query).await?;
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
    Ok(ok(strategy_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateStrategyRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(strategy_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateStrategyRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(strategy_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    strategy_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
