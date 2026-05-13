use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConfigItem {
    pub id: u64,
    pub config_type: String,
    pub name: String,
    pub display_name: String,
    pub is_builtin: bool,
    pub enabled: bool,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
