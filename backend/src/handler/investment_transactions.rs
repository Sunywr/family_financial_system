use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};

use crate::{
    common::{response::paged, state::AppState},
    dto::investment_transaction::InvestmentTransactionListQuery,
    error::app_error::AppError,
    service::{auth_service, investment_transaction_service},
};

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<InvestmentTransactionListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let (list, total) = investment_transaction_service::list(&state, &query, auth_user_id).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}
