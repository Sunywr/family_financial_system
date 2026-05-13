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
    dto::user::{CreateUserRequest, UpdateUserRequest, UserListQuery},
    error::app_error::AppError,
    service::user_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = user_service::list(&state, &query).await?;
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
    Ok(ok(user_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(user_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(user_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    user_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

pub async fn options(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(ok(user_service::options(&state).await?))
}
