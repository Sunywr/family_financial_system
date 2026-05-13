use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};

use crate::{
    common::{
        response::{ok, paged},
        state::AppState,
    },
    dto::investment::{InvestmentListQuery, InvestmentTopQuery},
    error::app_error::AppError,
    service::investment_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<InvestmentListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = investment_service::list(&state, &query).await?;
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
    Ok(ok(investment_service::detail(&state, id).await?))
}

pub async fn top(
    State(state): State<AppState>,
    Query(query): Query<InvestmentTopQuery>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(investment_service::top(&state, &query).await?))
}
