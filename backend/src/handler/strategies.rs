use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};

use crate::{
    common::{
        response::{ok, paged},
        state::AppState,
    },
    dto::strategy::{CreateStrategyRequest, StrategyListQuery, UpdateStrategyRequest},
    error::app_error::AppError,
    service::{auth_service, strategy_service},
};

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<StrategyListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let (list, total) = strategy_service::list(&state, &query, auth_user_id).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    Ok(ok(strategy_service::detail(&state, id, auth_user_id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateStrategyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    Ok(ok(strategy_service::create(&state, &payload, auth_user_id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateStrategyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    Ok(ok(
        strategy_service::update(&state, id, &payload, auth_user_id).await?
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    strategy_service::delete(&state, id, auth_user_id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
