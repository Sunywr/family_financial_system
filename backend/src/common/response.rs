use axum::{Json, response::IntoResponse};
use serde::Serialize;

use super::pagination::PageData;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data,
        }
    }
}

pub fn ok<T>(data: T) -> impl IntoResponse
where
    T: Serialize,
{
    Json(ApiResponse::success(data))
}

pub fn paged<T>(list: Vec<T>, total: u64, page: u64, page_size: u64) -> impl IntoResponse
where
    T: Serialize,
{
    ok(PageData {
        list,
        total,
        page,
        page_size,
    })
}
