use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Investment {
    pub id: u64,
    pub user_id: u64,
    pub source_bill_id: Option<u64>,
    pub investment_type: String,
    pub name: String,
    pub code: String,
    pub organization_name: String,
    pub market: Option<String>,
    pub total_shares: String,
    pub total_cost: String,
    pub average_cost: String,
    pub current_price: String,
    pub market_value: String,
    pub realized_profit: String,
    pub unrealized_profit: String,
    pub total_profit: String,
    pub total_profit_rate: String,
    pub latest_remark: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
