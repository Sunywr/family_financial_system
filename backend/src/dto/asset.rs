use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct AssetListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub user_id: u64,
    pub name: String,
    pub category_id: u64,
    pub amount: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssetRequest {
    pub name: String,
    pub category_id: u64,
    pub amount: String,
    pub remark: Option<String>,
    pub status: String,
}
