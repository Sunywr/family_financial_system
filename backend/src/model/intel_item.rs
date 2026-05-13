use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct IntelItem {
    pub id: u64,
    pub user_id: u64,
    pub title: String,
    pub source: Option<String>,
    pub item_date: NaiveDate,
    pub status: String,
    pub tags: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
