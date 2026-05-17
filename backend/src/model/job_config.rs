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

/// JobConfig enriched with the latest job_run info, used in the list endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct JobConfigSummary {
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
    pub last_run_status: Option<String>,
    pub last_run_started_at: Option<NaiveDateTime>,
    pub last_run_finished_at: Option<NaiveDateTime>,
    pub last_run_duration_ms: Option<u64>,
}
