use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};

use crate::{
    common::{response::ok, state::AppState},
    dto::auth::LoginRequest,
    error::app_error::AppError,
    service::auth_service,
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(auth_service::login(&state, &payload).await?))
}

pub async fn captcha(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(ok(auth_service::captcha(&state)?))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(auth_service::me(&state, &headers).await?))
}

pub async fn logout() -> Result<impl IntoResponse, AppError> {
    Ok(ok(auth_service::logout()))
}
