use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Deserialize)]
pub struct UpdateBillTagRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncLegacyBillTagsRequest {
    pub user_id: u64,
}

#[derive(Debug, Serialize)]
pub struct SyncLegacyBillTagsResponse {
    pub imported: u64,
    pub skipped: u64,
    pub source_total: u64,
    pub target_total: u64,
}
