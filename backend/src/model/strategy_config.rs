use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StrategyConfig {
    pub id: u64,
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
