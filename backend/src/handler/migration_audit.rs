use axum::{extract::State, response::IntoResponse};

use crate::{
    common::{response::ok, state::AppState},
    error::app_error::AppError,
    service::migration_audit_service,
};

pub async fn summary(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(ok(migration_audit_service::build_summary(&state).await?))
}
