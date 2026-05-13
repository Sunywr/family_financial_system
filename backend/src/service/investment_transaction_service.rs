use crate::{
    common::state::AppState, dto::investment_transaction::InvestmentTransactionListQuery,
    error::app_error::AppError, model::investment_transaction::InvestmentTransaction,
    repository::investment_transaction_repository,
};

pub async fn list(
    state: &AppState,
    query: &InvestmentTransactionListQuery,
) -> Result<(Vec<InvestmentTransaction>, u64), AppError> {
    investment_transaction_repository::list(state.db()?, query).await
}
