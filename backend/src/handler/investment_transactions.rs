use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    common::{response::paged, state::AppState},
    dto::investment_transaction::InvestmentTransactionListQuery,
    error::app_error::AppError,
    service::investment_transaction_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<InvestmentTransactionListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = investment_transaction_service::list(&state, &query).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}
