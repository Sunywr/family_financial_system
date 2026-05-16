use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::investment::{InvestmentListQuery, InvestmentTopQuery, UpdateInvestmentRequest},
    error::app_error::AppError,
    model::{investment::Investment, investment_recommendation::InvestmentRecommendation},
    repository::{investment_repository, user_repository},
    service::scoring_service,
};

pub async fn list(
    state: &AppState,
    query: &InvestmentListQuery,
    auth_user_id: u64,
) -> Result<(Vec<Investment>, u64), AppError> {
    let pool = state.db()?;
    let auth_user = user_repository::find_by_id(pool, auth_user_id).await?;
    let effective_query = InvestmentListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        investment_type: query.investment_type.clone(),
        status: query.status.clone(),
        show_sold: query.show_sold,
        keyword: query.keyword.clone(),
    };
    investment_repository::list(pool, &effective_query).await
}

pub async fn detail(state: &AppState, id: u64, auth_user_id: u64) -> Result<Investment, AppError> {
    let investment = investment_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, investment.user_id).await?;
    Ok(investment)
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateInvestmentRequest,
    auth_user_id: u64,
) -> Result<Investment, AppError> {
    let pool = state.db()?;
    let auth_user = user_repository::find_by_id(pool, auth_user_id).await?;
    let investment = investment_repository::find_by_id(pool, id).await?;

    if auth_user.username != "admin" && investment.user_id != auth_user_id {
        return Err(AppError::Unauthorized);
    }

    let current_price = parse_non_negative_decimal(&payload.current_price, "current_price")?;
    let market_value = parse_non_negative_decimal(&payload.market_value, "market_value")?;
    let total_shares = parse_non_negative_decimal(&payload.total_shares, "total_shares")?;
    let total_cost = parse_non_negative_decimal(&payload.total_cost, "total_cost")?;
    let realized_profit =
        parse_non_negative_decimal(&investment.realized_profit, "realized_profit")?;

    let average_cost = if total_shares > Decimal::ZERO {
        total_cost / total_shares
    } else {
        Decimal::ZERO
    };
    let unrealized_profit = market_value - total_cost;
    let total_profit = realized_profit + unrealized_profit;
    let total_profit_rate = if total_cost > Decimal::ZERO {
        total_profit / total_cost
    } else {
        Decimal::ZERO
    };
    let status = if total_shares > Decimal::ZERO {
        "holding"
    } else {
        "sold"
    };

    investment_repository::update_manual_metrics(
        pool,
        id,
        &format_decimal6(total_shares),
        &format_decimal2(total_cost),
        &format_decimal6(average_cost),
        &format_decimal6(current_price),
        &format_decimal2(market_value),
        &format_decimal2(unrealized_profit),
        &format_decimal2(total_profit),
        &format_decimal6(total_profit_rate),
        status,
    )
    .await?;

    detail(state, id, auth_user_id).await
}

pub async fn top(
    state: &AppState,
    query: &InvestmentTopQuery,
    auth_user_id: u64,
) -> Result<Vec<InvestmentRecommendation>, AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = InvestmentTopQuery {
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        investment_type: query.investment_type.clone(),
        limit: query.limit,
    };

    let mut list = scoring_service::list_top_recommendations(state, &effective_query).await?;
    if list.is_empty() {
        match effective_query.investment_type.as_deref() {
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
        list = scoring_service::list_top_recommendations(state, &effective_query).await?;
    }
    Ok(list)
}

async fn ensure_owner_access(
    state: &AppState,
    auth_user_id: u64,
    owner_user_id: u64,
) -> Result<(), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    if auth_user.username != "admin" && owner_user_id != auth_user_id {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}

fn parse_non_negative_decimal(value: &str, field: &str) -> Result<Decimal, AppError> {
    let decimal = Decimal::from_str(value)
        .map_err(|_| AppError::BadRequest(format!("{field} must be a valid decimal string")))?;
    if decimal < Decimal::ZERO {
        return Err(AppError::BadRequest(format!(
            "{field} must not be negative"
        )));
    }
    Ok(decimal)
}

fn format_decimal2(value: Decimal) -> String {
    format!("{value:.2}")
}

fn format_decimal6(value: Decimal) -> String {
    format!("{value:.6}")
}
