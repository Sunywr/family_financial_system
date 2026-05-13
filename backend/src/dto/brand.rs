use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct BrandListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub board_type: Option<String>,
    pub category_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBrandRequest {
    pub user_id: u64,
    pub category_id: u64,
    pub brand_name: String,
    pub score: i32,
    pub board_type: String,
    pub review: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBrandRequest {
    pub category_id: u64,
    pub brand_name: String,
    pub score: i32,
    pub board_type: String,
    pub review: Option<String>,
    pub remark: Option<String>,
}
