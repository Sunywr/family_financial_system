use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JobConfig {
    pub id: u64,
    pub job_code: String,
    pub job_name: String,
    pub cron_expr: String,
    pub enabled: bool,
    pub batch_size: u32,
    pub concurrency: u32,
    pub timeout_seconds: u32,
    pub retry_count: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
