use crate::{
    common::state::AppState, dto::investment_transaction::InvestmentTransactionListQuery,
    error::app_error::AppError, model::investment_transaction::InvestmentTransaction,
    repository::{investment_transaction_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &InvestmentTransactionListQuery,
    auth_user_id: u64,
) -> Result<(Vec<InvestmentTransaction>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = InvestmentTransactionListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        investment_id: query.investment_id,
        source_bill_id: query.source_bill_id,
        keyword: query.keyword.clone(),
    };
    investment_transaction_repository::list(state.db()?, &effective_query).await
}
