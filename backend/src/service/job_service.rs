use std::{str::FromStr, time::Instant};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, NaiveDateTime, Utc};
use cron::Schedule;

use crate::{
    common::state::AppState,
    dto::{
        dashboard::DashboardSummaryQuery,
        job::{JobListQuery, JobRunListQuery, TriggerJobRequest, UpdateJobRequest},
    },
    error::app_error::AppError,
    model::{job_config::JobConfig, job_run::JobRun},
    repository::{job_repository, user_repository},
    service::{dashboard_service, scoring_service},
};

pub async fn seed_jobs(state: &AppState) -> Result<(), AppError> {
    job_repository::seed_jobs(state.db()?).await
}

pub async fn list_jobs(
    state: &AppState,
    query: &JobListQuery,
) -> Result<(Vec<JobConfig>, u64), AppError> {
    job_repository::list_jobs(state.db()?, query).await
}

pub async fn detail_job(state: &AppState, id: u64) -> Result<JobConfig, AppError> {
    job_repository::find_job_by_id(state.db()?, id).await
}

pub async fn update_job(
    state: &AppState,
    id: u64,
    payload: &UpdateJobRequest,
) -> Result<JobConfig, AppError> {
    validate_schedule(&payload.cron_expr)?;
    job_repository::update_job(state.db()?, id, payload).await?;
    detail_job(state, id).await
}

pub async fn list_runs(
    state: &AppState,
    query: &JobRunListQuery,
) -> Result<(Vec<JobRun>, u64), AppError> {
    job_repository::list_job_runs(state.db()?, query).await
}

pub async fn run_due_jobs(state: &AppState) -> Result<(), AppError> {
    let now = Utc::now();
    let jobs = job_repository::list_enabled_jobs(state.db()?).await?;
    for job in jobs {
        if let Some(scheduled_at) = latest_due_schedule(&job.cron_expr, now.naive_utc())? {
            let already_ran =
                job_repository::has_run_for_schedule(state.db()?, job.id, scheduled_at).await?;
            if !already_ran {
                let _ = execute_job(state, &job, Some(scheduled_at), "scheduler").await;
            }
        }
    }
    Ok(())
}

pub async fn trigger_job(
    state: &AppState,
    id: u64,
    payload: &TriggerJobRequest,
) -> Result<JobRun, AppError> {
    let job = detail_job(state, id).await?;
    let run_id = execute_job(
        state,
        &job,
        None,
        payload.trigger_type.as_deref().unwrap_or("manual"),
    )
    .await?;
    let (runs, _) = job_repository::list_job_runs(
        state.db()?,
        &JobRunListQuery {
            pagination: crate::common::pagination::PaginationQuery {
                page: 1,
                page_size: 1,
            },
            job_id: Some(job.id),
            status: None,
        },
    )
    .await?;
    let _ = run_id;
    runs.into_iter().next().ok_or(AppError::Internal)
}

async fn execute_job(
    state: &AppState,
    job: &JobConfig,
    scheduled_at: Option<NaiveDateTime>,
    trigger_type: &str,
) -> Result<u64, AppError> {
    let start = Instant::now();
    let run_id = job_repository::create_job_run(
        state.db()?,
        job.id,
        "running",
        scheduled_at,
        trigger_type,
        Some("job started"),
    )
    .await?;

    let result = match job.job_code.as_str() {
        "dashboard_snapshot_daily" => run_dashboard_snapshot_job(state).await,
        "stock_market_sync_daily" => run_stock_scoring_job(state).await,
        "wealth_sync_daily_slots" => run_wealth_scoring_job(state).await,
        "stock_realtime_sync" => run_stock_realtime_job(state).await,
        _ => Err(AppError::BadRequest("unsupported job code".to_string())),
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    match result {
        Ok(message) => {
            job_repository::finish_job_run(
                state.db()?,
                run_id,
                "success",
                Some(&message),
                None,
                duration_ms,
            )
            .await?;
        }
        Err(error) => {
            job_repository::finish_job_run(
                state.db()?,
                run_id,
                "failed",
                Some("job failed"),
                Some(&error.to_string()),
                duration_ms,
            )
            .await?;
        }
    }

    Ok(run_id)
}

async fn run_dashboard_snapshot_job(state: &AppState) -> Result<String, AppError> {
    let pool = state.db()?;
    let users = user_repository::list_options(pool).await?;
    let today = Utc::now().date_naive();

    for user in users {
        let month_start =
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1).ok_or(AppError::Internal)?;
        let summary = dashboard_service::summary(
            state,
            &DashboardSummaryQuery {
                user_id: user.id,
                start_date: month_start,
                end_date: today,
            },
        )
        .await?;
        job_repository::upsert_dashboard_snapshot(
            pool,
            user.id,
            today,
            &summary.cash_balance,
            &summary.total_assets,
            &summary.outstanding_amount,
            &summary.wealth_amount,
            &summary.stock_amount,
            &summary.income,
            &summary.expense,
            &summary.net_cash_flow,
            &summary.calibration_status,
        )
        .await?;
    }

    Ok("dashboard snapshot completed".to_string())
}

async fn run_stock_scoring_job(state: &AppState) -> Result<String, AppError> {
    let affected = scoring_service::refresh_stock_scores(state).await?;
    Ok(format!("stock scores refreshed: {affected}"))
}

async fn run_wealth_scoring_job(state: &AppState) -> Result<String, AppError> {
    let affected = scoring_service::refresh_wealth_scores(state).await?;
    Ok(format!("wealth scores refreshed: {affected}"))
}

async fn run_stock_realtime_job(state: &AppState) -> Result<String, AppError> {
    let affected = scoring_service::refresh_stock_scores(state).await?;
    Ok(format!("realtime stock scores refreshed: {affected}"))
}

fn validate_schedule(expr: &str) -> Result<(), AppError> {
    Schedule::from_str(expr)
        .map(|_| ())
        .map_err(|_| AppError::BadRequest("invalid cron_expr".to_string()))
}

fn latest_due_schedule(expr: &str, now: NaiveDateTime) -> Result<Option<NaiveDateTime>, AppError> {
    let schedule = Schedule::from_str(expr)
        .map_err(|_| AppError::BadRequest("invalid cron_expr".to_string()))?;
    let now_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(now, Utc);
    let lookback = now_utc - ChronoDuration::minutes(10);
    let due = schedule
        .after(&lookback)
        .take(32)
        .filter(|dt| *dt <= now_utc)
        .last();
    Ok(due.map(|dt| dt.naive_utc()))
}
