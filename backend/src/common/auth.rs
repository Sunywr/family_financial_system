use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{common::state::AppState, error::app_error::AppError, service::auth_service};

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    auth_service::authenticate_request(&state, request.headers()).await?;
    Ok(next.run(request).await)
}
