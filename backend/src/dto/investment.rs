use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct InvestmentListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub investment_type: Option<String>,
    pub status: Option<String>,
    pub show_sold: Option<bool>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InvestmentTopQuery {
    pub user_id: Option<u64>,
    pub investment_type: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvestmentRequest {
    pub current_price: String,
    pub market_value: String,
    pub total_shares: String,
    pub total_cost: String,
}
