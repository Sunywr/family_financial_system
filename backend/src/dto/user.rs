use serde::{Deserialize, Serialize};

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub keyword: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub role: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: String,
    pub role: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct UserOptionsDto {
    pub id: u64,
    pub display_name: String,
}
