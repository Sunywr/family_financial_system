use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DashboardSummaryQuery {
    pub user_id: u64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
