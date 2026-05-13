use crate::{
    common::state::AppState,
    dto::investment::{InvestmentListQuery, InvestmentTopQuery},
    error::app_error::AppError,
    model::{investment::Investment, investment_recommendation::InvestmentRecommendation},
    repository::investment_repository,
    service::scoring_service,
};

pub async fn list(
    state: &AppState,
    query: &InvestmentListQuery,
) -> Result<(Vec<Investment>, u64), AppError> {
    investment_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<Investment, AppError> {
    investment_repository::find_by_id(state.db()?, id).await
}

pub async fn top(
    state: &AppState,
    query: &InvestmentTopQuery,
) -> Result<Vec<InvestmentRecommendation>, AppError> {
    let mut list = scoring_service::list_top_recommendations(state, query).await?;
    if list.is_empty() {
        match query.investment_type.as_deref() {
            Some("stock") => {
                let _ = scoring_service::refresh_stock_scores(state).await?;
            }
            Some("wealth") => {
                let _ = scoring_service::refresh_wealth_scores(state).await?;
            }
            _ => {
                let _ = scoring_service::refresh_stock_scores(state).await?;
                let _ = scoring_service::refresh_wealth_scores(state).await?;
            }
        }
        list = scoring_service::list_top_recommendations(state, query).await?;
    }
    Ok(list)
}
