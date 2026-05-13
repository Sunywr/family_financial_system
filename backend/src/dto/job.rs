use serde::Deserialize;

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct JobListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JobRunListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub job_id: Option<u64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobRequest {
    pub cron_expr: String,
    pub enabled: bool,
    pub batch_size: u32,
    pub concurrency: u32,
    pub timeout_seconds: u32,
    pub retry_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct TriggerJobRequest {
    pub trigger_type: Option<String>,
}
