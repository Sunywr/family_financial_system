use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JobRun {
    pub id: u64,
    pub job_id: u64,
    pub status: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub trigger_type: String,
    pub started_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
    pub duration_ms: Option<u64>,
    pub message: Option<String>,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
