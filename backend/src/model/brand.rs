use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Brand {
    pub id: u64,
    pub user_id: u64,
    pub category_id: u64,
    pub category_name: String,
    pub brand_name: String,
    pub score: i32,
    pub board_type: String,
    pub review: Option<String>,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
