use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct StrategyListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub investment_type: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStrategyRequest {
    pub user_id: u64,
    pub investment_type: String,
    pub target_code: Option<String>,
    pub strategy_name: String,
    pub enabled: bool,
    pub risk_level: String,
    pub preferred_min_score: i32,
    pub cooldown_days: i32,
    pub take_profit_rate: String,
    pub stop_loss_rate: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStrategyRequest {
    pub investment_type: String,
    pub target_code: Option<String>,
    pub strategy_name: String,
    pub enabled: bool,
    pub risk_level: String,
    pub preferred_min_score: i32,
    pub cooldown_days: i32,
    pub take_profit_rate: String,
    pub stop_loss_rate: String,
    pub notes: Option<String>,
}
