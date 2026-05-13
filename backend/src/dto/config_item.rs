use serde::{Deserialize, Serialize};

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct ConfigItemListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub config_type: Option<String>,
    pub keyword: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConfigItemRequest {
    pub config_type: String,
    pub name: String,
    pub display_name: String,
    pub enabled: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigItemRequest {
    pub display_name: String,
    pub enabled: bool,
    pub sort_order: i32,
}

#[derive(Debug, Serialize)]
pub struct ConfigTypeDto {
    pub code: &'static str,
    pub label: &'static str,
}
