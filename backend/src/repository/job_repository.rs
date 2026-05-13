use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::job::{JobListQuery, JobRunListQuery, UpdateJobRequest},
    error::app_error::AppError,
    model::{dashboard_snapshot::DashboardSnapshot, job_config::JobConfig, job_run::JobRun},
};

fn map_job(row: sqlx::mysql::MySqlRow) -> Result<JobConfig, sqlx::Error> {
    Ok(JobConfig {
        id: row.try_get("id")?,
        job_code: row.try_get("job_code")?,
        job_name: row.try_get("job_name")?,
        cron_expr: row.try_get("cron_expr")?,
        enabled: row.try_get("enabled")?,
        batch_size: row.try_get("batch_size")?,
        concurrency: row.try_get("concurrency")?,
        timeout_seconds: row.try_get("timeout_seconds")?,
        retry_count: row.try_get("retry_count")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn map_job_run(row: sqlx::mysql::MySqlRow) -> Result<JobRun, sqlx::Error> {
    Ok(JobRun {
        id: row.try_get("id")?,
        job_id: row.try_get("job_id")?,
        status: row.try_get("status")?,
        scheduled_at: row.try_get("scheduled_at")?,
        trigger_type: row.try_get("trigger_type")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        duration_ms: row.try_get("duration_ms")?,
        message: row.try_get("message")?,
        error_message: row.try_get("error_message")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn map_snapshot(row: sqlx::mysql::MySqlRow) -> Result<DashboardSnapshot, sqlx::Error> {
    Ok(DashboardSnapshot {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        snapshot_date: row.try_get::<NaiveDate, _>("snapshot_date")?,
        cash_balance: row.try_get("cash_balance")?,
        total_assets: row.try_get("total_assets")?,
        outstanding_amount: row.try_get("outstanding_amount")?,
        wealth_amount: row.try_get("wealth_amount")?,
        stock_amount: row.try_get("stock_amount")?,
        income: row.try_get("income")?,
        expense: row.try_get("expense")?,
        net_cash_flow: row.try_get("net_cash_flow")?,
        calibration_status: row.try_get("calibration_status")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn seed_jobs(pool: &sqlx::MySqlPool) -> Result<(), AppError> {
    let jobs = [
        ("stock_market_sync_daily", "全市场股票同步", "0 0 1 * * * *"),
        ("wealth_sync_daily_slots", "理财产品同步", "0 0 */6 * * * *"),
        ("stock_realtime_sync", "交易时段股票同步", "0 */5 * * * * *"),
        ("dashboard_snapshot_daily", "首页快照", "0 10 23 * * * *"),
    ];

    for (job_code, job_name, cron_expr) in jobs {
        sqlx::query(
            "INSERT INTO job_configs (job_code, job_name, cron_expr, enabled, batch_size, concurrency, timeout_seconds, retry_count)
             VALUES (?, ?, ?, 1, 100, 1, 300, 0)
             ON DUPLICATE KEY UPDATE job_name = VALUES(job_name), cron_expr = VALUES(cron_expr)",
        )
        .bind(job_code)
        .bind(job_name)
        .bind(cron_expr)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn list_jobs(
    pool: &sqlx::MySqlPool,
    query: &JobListQuery,
) -> Result<(Vec<JobConfig>, u64), AppError> {
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM job_configs
         WHERE deleted_at IS NULL
           AND (? IS NULL OR enabled = ?)",
    )
    .bind(query.enabled)
    .bind(query.enabled)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, job_code, job_name, cron_expr, enabled, batch_size, concurrency, timeout_seconds, retry_count, created_at, updated_at
         FROM job_configs
         WHERE deleted_at IS NULL
           AND (? IS NULL OR enabled = ?)
         ORDER BY id ASC
         LIMIT ? OFFSET ?",
    )
    .bind(query.enabled)
    .bind(query.enabled)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_job)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_job_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<JobConfig, AppError> {
    let row = sqlx::query(
        "SELECT id, job_code, job_name, cron_expr, enabled, batch_size, concurrency, timeout_seconds, retry_count, created_at, updated_at
         FROM job_configs
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_job).transpose()?.ok_or(AppError::NotFound)
}

pub async fn list_enabled_jobs(pool: &sqlx::MySqlPool) -> Result<Vec<JobConfig>, AppError> {
    let rows = sqlx::query(
        "SELECT id, job_code, job_name, cron_expr, enabled, batch_size, concurrency, timeout_seconds, retry_count, created_at, updated_at
         FROM job_configs
         WHERE enabled = 1 AND deleted_at IS NULL
         ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(map_job)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

pub async fn update_job(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateJobRequest,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE job_configs
         SET cron_expr = ?, enabled = ?, batch_size = ?, concurrency = ?, timeout_seconds = ?, retry_count = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.cron_expr)
    .bind(payload.enabled)
    .bind(payload.batch_size)
    .bind(payload.concurrency)
    .bind(payload.timeout_seconds)
    .bind(payload.retry_count)
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn create_job_run(
    pool: &sqlx::MySqlPool,
    job_id: u64,
    status: &str,
    scheduled_at: Option<NaiveDateTime>,
    trigger_type: &str,
    message: Option<&str>,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO job_runs (job_id, status, scheduled_at, trigger_type, started_at, message)
         VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, ?)",
    )
    .bind(parse_u64_id(job_id)?)
    .bind(status)
    .bind(scheduled_at)
    .bind(trigger_type)
    .bind(message)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn finish_job_run(
    pool: &sqlx::MySqlPool,
    run_id: u64,
    status: &str,
    message: Option<&str>,
    error_message: Option<&str>,
    duration_ms: u64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE job_runs
         SET status = ?, finished_at = CURRENT_TIMESTAMP, duration_ms = ?, message = ?, error_message = ?
         WHERE id = ?",
    )
    .bind(status)
    .bind(duration_ms)
    .bind(message)
    .bind(error_message)
    .bind(parse_u64_id(run_id)?)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn has_run_for_schedule(
    pool: &sqlx::MySqlPool,
    job_id: u64,
    scheduled_at: NaiveDateTime,
) -> Result<bool, AppError> {
    let row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM job_runs
         WHERE job_id = ? AND scheduled_at = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(job_id)?)
    .bind(scheduled_at)
    .fetch_one(pool)
    .await?;
    let total = row.try_get::<i64, _>("total")?.max(0) as u64;
    Ok(total > 0)
}

pub async fn list_job_runs(
    pool: &sqlx::MySqlPool,
    query: &JobRunListQuery,
) -> Result<(Vec<JobRun>, u64), AppError> {
    let job_id = query.job_id.map(parse_u64_id).transpose()?;
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM job_runs
         WHERE deleted_at IS NULL
           AND (? IS NULL OR job_id = ?)
           AND (? IS NULL OR status = ?)",
    )
    .bind(job_id)
    .bind(job_id)
    .bind(&query.status)
    .bind(&query.status)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, job_id, status, scheduled_at, trigger_type, started_at, finished_at, duration_ms, message, error_message, created_at, updated_at
         FROM job_runs
         WHERE deleted_at IS NULL
           AND (? IS NULL OR job_id = ?)
           AND (? IS NULL OR status = ?)
         ORDER BY started_at DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(job_id)
    .bind(job_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_job_run)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_dashboard_snapshot(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    snapshot_date: NaiveDate,
    cash_balance: &str,
    total_assets: &str,
    outstanding_amount: &str,
    wealth_amount: &str,
    stock_amount: &str,
    income: &str,
    expense: &str,
    net_cash_flow: &str,
    calibration_status: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO dashboard_snapshots (
            user_id, snapshot_date, cash_balance, total_assets, outstanding_amount, wealth_amount,
            stock_amount, income, expense, net_cash_flow, calibration_status
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
           cash_balance = VALUES(cash_balance),
           total_assets = VALUES(total_assets),
           outstanding_amount = VALUES(outstanding_amount),
           wealth_amount = VALUES(wealth_amount),
           stock_amount = VALUES(stock_amount),
           income = VALUES(income),
           expense = VALUES(expense),
           net_cash_flow = VALUES(net_cash_flow),
           calibration_status = VALUES(calibration_status)",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(snapshot_date)
    .bind(cash_balance)
    .bind(total_assets)
    .bind(outstanding_amount)
    .bind(wealth_amount)
    .bind(stock_amount)
    .bind(income)
    .bind(expense)
    .bind(net_cash_flow)
    .bind(calibration_status)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn latest_dashboard_snapshots(
    pool: &sqlx::MySqlPool,
    limit: u64,
) -> Result<Vec<DashboardSnapshot>, AppError> {
    let rows = sqlx::query(
        "SELECT id, user_id, snapshot_date, CAST(cash_balance AS CHAR) AS cash_balance,
                CAST(total_assets AS CHAR) AS total_assets,
                CAST(outstanding_amount AS CHAR) AS outstanding_amount,
                CAST(wealth_amount AS CHAR) AS wealth_amount,
                CAST(stock_amount AS CHAR) AS stock_amount,
                CAST(income AS CHAR) AS income,
                CAST(expense AS CHAR) AS expense,
                CAST(net_cash_flow AS CHAR) AS net_cash_flow,
                calibration_status, created_at, updated_at
         FROM dashboard_snapshots
         WHERE deleted_at IS NULL
         ORDER BY snapshot_date DESC, id DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(map_snapshot)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}
