use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct InvestmentTransactionListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub investment_id: Option<u64>,
    pub source_bill_id: Option<u64>,
    pub keyword: Option<String>,
}
