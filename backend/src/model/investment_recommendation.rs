use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct InvestmentRecommendation {
    pub investment_id: u64,
    pub user_id: u64,
    pub investment_type: String,
    pub name: String,
    pub code: String,
    pub organization_name: String,
    pub current_price: String,
    pub market_value: String,
    pub total_profit_rate: String,
    pub score: i32,
    pub suggestion: String,
    pub reason: String,
    pub indicator_date: NaiveDate,
}
