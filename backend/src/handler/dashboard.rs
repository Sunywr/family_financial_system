use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    common::{response::ok, state::AppState},
    dto::dashboard::DashboardSummaryQuery,
    error::app_error::AppError,
    service::dashboard_service,
};

pub async fn summary(
    State(state): State<AppState>,
    Query(query): Query<DashboardSummaryQuery>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(dashboard_service::summary(&state, &query).await?))
}

pub async fn cash_trend(
    State(state): State<AppState>,
    Query(query): Query<DashboardSummaryQuery>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(dashboard_service::cash_trend(&state, &query).await?))
}
