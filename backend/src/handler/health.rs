use axum::{extract::State, response::IntoResponse};

use crate::{
    common::{response::ok, state::AppState},
    error::app_error::AppError,
    service::health_service,
};

pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let data = health_service::build_health(&state).await?;
    Ok(ok(data))
}
