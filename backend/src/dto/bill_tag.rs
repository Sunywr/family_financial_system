use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct BillTagListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBillTagRequest {
    pub user_id: u64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBillTagRequest {
    pub name: String,
    pub color: Option<String>,
}
