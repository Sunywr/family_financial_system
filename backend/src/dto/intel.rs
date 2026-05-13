use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct IntelListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIntelRequest {
    pub user_id: u64,
    pub title: String,
    pub source: Option<String>,
    pub item_date: NaiveDate,
    pub status: String,
    pub tags: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIntelRequest {
    pub title: String,
    pub source: Option<String>,
    pub item_date: NaiveDate,
    pub status: String,
    pub tags: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
}
