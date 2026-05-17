use std::{str::FromStr, time::Instant};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, NaiveDateTime, Utc};
use cron::Schedule;
use rust_decimal::Decimal;
use sqlx::Row;

use crate::{
    common::state::AppState,
    dto::{
        dashboard::DashboardSummaryQuery,
        job::{JobListQuery, JobRunListQuery, TriggerJobRequest, UpdateJobRequest},
    },
    error::app_error::AppError,
    model::{job_config::{JobConfig, JobConfigSummary}, job_run::JobRun},
    repository::{job_repository, user_repository},
    service::{auto_invest_plan_service, budget_service, dashboard_service, scoring_service},
};

pub async fn seed_jobs(state: &AppState) -> Result<(), AppError> {
    job_repository::seed_jobs(state.db()?).await
}

pub async fn list_jobs(
    state: &AppState,
    query: &JobListQuery,
) -> Result<(Vec<JobConfigSummary>, u64), AppError> {
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
        "debt_cycle_bill_generate_daily" => run_debt_cycle_bill_generate_job(state).await,
        "stock_market_sync_daily" => run_stock_scoring_job(state).await,
        "wealth_sync_daily_slots" => run_wealth_scoring_job(state).await,
        "stock_realtime_sync" => run_stock_realtime_job(state).await,
        "budget_generate_monthly" => run_budget_generate_job(state).await,
        "auto_invest_generate_daily" => run_auto_invest_generate_job(state).await,
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

#[derive(Debug)]
struct ScheduledDebtBillRow {
    id: u64,
    user_id: u64,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    repay_deadline: Option<NaiveDate>,
    category_name: String,
    category_id: u64,
    amount: Decimal,
    period_count: u32,
    period_unit: String,
    period_value: u32,
    payment_method: String,
    remark: Option<String>,
    source_credit_card_id: Option<u64>,
}

fn month_last_day(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("valid month");
    (next_month - ChronoDuration::days(1)).day()
}

fn shift_date(date: NaiveDate, unit: &str, value: u32) -> Option<NaiveDate> {
    let step = value.max(1);
    match unit {
        "day" => date.checked_add_signed(ChronoDuration::days(i64::from(step))),
        "month" => {
            let total_months = date.year() * 12 + date.month0() as i32 + step as i32;
            let year = total_months.div_euclid(12);
            let month0 = total_months.rem_euclid(12) as u32;
            let month = month0 + 1;
            let day = date.day().min(month_last_day(year, month));
            NaiveDate::from_ymd_opt(year, month, day)
        }
        "year" => {
            let year = date.year() + step as i32;
            let day = date.day().min(month_last_day(year, date.month()));
            NaiveDate::from_ymd_opt(year, date.month(), day)
        }
        _ => None,
    }
}

fn build_due_dates(row: &ScheduledDebtBillRow) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut cursor = row.start_date;
    for _ in 0..row.period_count.max(1) {
        if let Some(end_date) = row.end_date
            && cursor > end_date
        {
            break;
        }
        if let Some(repay_deadline) = row.repay_deadline
            && cursor > repay_deadline
        {
            break;
        }
        dates.push(cursor);
        let Some(next) = shift_date(cursor, &row.period_unit, row.period_value) else {
            break;
        };
        if next <= cursor {
            break;
        }
        cursor = next;
    }
    dates
}

fn format_money(value: Decimal) -> String {
    format!("{:.2}", value.round_dp(2))
}

async fn run_debt_cycle_bill_generate_job(state: &AppState) -> Result<String, AppError> {
    let pool = state.db()?;
    let today = Utc::now().date_naive();
    let cleaned_non_credit = sqlx::query(
        "UPDATE bills b
         LEFT JOIN debts d
                ON d.id = b.related_debt_id
               AND d.deleted_at IS NULL
         SET b.deleted_at = CURRENT_TIMESTAMP,
             b.updated_at = CURRENT_TIMESTAMP
         WHERE b.related_debt_id IS NOT NULL
           AND b.special_status = 'debt_cycle_auto'
           AND b.deleted_at IS NULL
           AND (d.id IS NULL OR d.payment_method <> 'credit_card')",
    )
    .execute(pool)
    .await?
    .rows_affected();

    let rows = sqlx::query(
        "SELECT d.id,
                d.user_id,
                d.start_date,
                d.end_date,
                d.repay_deadline,
                d.category_id,
                d.category_name,
                CAST(d.amount AS CHAR) AS amount,
                d.period_count,
                d.period_unit,
                d.period_value,
                d.payment_method,
                d.remark,
                b.credit_card_id AS source_credit_card_id
         FROM debts d
         LEFT JOIN bills b
                ON b.id = d.source_bill_id
               AND b.deleted_at IS NULL
         WHERE d.status = 'pending'
           AND d.deleted_at IS NULL
           AND d.start_date <= ?
                     AND d.payment_method = 'credit_card'",
    )
    .bind(today)
    .fetch_all(pool)
    .await?;

    let mut created_count = 0_u64;

    for raw in rows {
        let amount = raw
            .try_get::<String, _>("amount")?
            .parse::<Decimal>()
            .unwrap_or(Decimal::ZERO);
        if amount <= Decimal::ZERO {
            continue;
        }

        let debt = ScheduledDebtBillRow {
            id: raw.try_get("id")?,
            user_id: raw.try_get("user_id")?,
            start_date: raw.try_get("start_date")?,
            end_date: raw.try_get("end_date")?,
            repay_deadline: raw.try_get("repay_deadline")?,
            category_id: raw.try_get("category_id")?,
            category_name: raw.try_get("category_name")?,
            amount,
            period_count: raw.try_get::<u32, _>("period_count")?.max(1),
            period_unit: raw.try_get("period_unit")?,
            period_value: raw.try_get::<u32, _>("period_value")?.max(1),
            payment_method: raw.try_get("payment_method")?,
            remark: raw.try_get("remark")?,
            source_credit_card_id: raw.try_get("source_credit_card_id")?,
        };

        let due_dates = build_due_dates(&debt);
        if due_dates.is_empty() {
            continue;
        }

        let total_occurrences = Decimal::from(due_dates.len() as u64);
        let per_amount = (debt.amount / total_occurrences).round_dp(2);

        for (index, due_date) in due_dates.iter().enumerate() {
            if *due_date > today {
                break;
            }

            let existing = sqlx::query(
                "SELECT id
                 FROM bills
                 WHERE related_debt_id = ?
                   AND account_date = ?
                   AND bill_type = 'expense'
                   AND deleted_at IS NULL
                 LIMIT 1",
            )
            .bind(debt.id)
            .bind(*due_date)
            .fetch_optional(pool)
            .await?;
            if existing.is_some() {
                continue;
            }

            let cycle_amount = if index + 1 == due_dates.len() {
                (debt.amount - per_amount * Decimal::from((due_dates.len() - 1) as u64)).round_dp(2)
            } else {
                per_amount
            };

            let auto_remark = match debt.remark.as_deref() {
                Some(remark) if !remark.trim().is_empty() => {
                    format!("[cycle debt auto] debt_id={} {}", debt.id, remark.trim())
                }
                _ => format!("[cycle debt auto] debt_id={}", debt.id),
            };

            sqlx::query(
                "INSERT INTO bills (
                    user_id, account_date, category_id, category_name, bill_type, payment_method,
                    is_fixed_asset, amount, tags, remark, transfer_group_id, transfer_target_type,
                    transfer_target_user_id, credit_card_id, is_installment, installment_months,
                    investment_action, related_investment_id, product_code, product_name,
                    organization_name, share_amount, related_asset_id, related_debt_id, special_status
                 ) VALUES (
                    ?, ?, ?, ?, 'expense', ?,
                    0, ?, CAST(? AS JSON), ?, NULL, NULL,
                    NULL, ?, 0, NULL,
                    NULL, NULL, NULL, NULL,
                    NULL, NULL, NULL, ?, 'debt_cycle_auto'
                 )",
            )
            .bind(debt.user_id)
            .bind(*due_date)
            .bind(debt.category_id)
            .bind(&debt.category_name)
            .bind(&debt.payment_method)
            .bind(format_money(cycle_amount))
            .bind::<Option<String>>(None)
            .bind(auto_remark)
            .bind(debt.source_credit_card_id)
            .bind(debt.id)
            .execute(pool)
            .await?;
            created_count += 1;
        }
    }

    Ok(format!(
        "debt cycle bills generated: {created_count}, cleaned legacy non-credit auto bills: {cleaned_non_credit}"
    ))
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

async fn run_auto_invest_generate_job(state: &AppState) -> Result<String, AppError> {
    let pool = state.db()?;
    let today = Utc::now().date_naive();
    let (visited, created) =
        auto_invest_plan_service::generate_for_scheduler(pool, today).await?;
    Ok(format!(
        "auto_invest generate completed: {created} bills created, {visited} plans visited"
    ))
}

async fn run_budget_generate_job(state: &AppState) -> Result<String, AppError> {
    let pool = state.db()?;
    let today = Utc::now().date_naive();
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .ok_or(AppError::Internal)?;
    let users = user_repository::list_options(pool).await?;
    let user_count = users.len();
    let mut total_affected = 0usize;
    for user in users {
        let affected =
            budget_service::generate_for_scheduler(pool, user.id, month_start).await?;
        total_affected += affected;
    }
    Ok(format!(
        "budget generate completed: {total_affected} entries for {user_count} users"
    ))
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

#[cfg(test)]
mod tests {
    use super::{ScheduledDebtBillRow, build_due_dates};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn row(
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        repay_deadline: Option<NaiveDate>,
        period_count: u32,
        period_unit: &str,
        period_value: u32,
    ) -> ScheduledDebtBillRow {
        ScheduledDebtBillRow {
            id: 1,
            user_id: 1,
            start_date,
            end_date,
            repay_deadline,
            category_id: 1,
            category_name: "loan".to_string(),
            amount: Decimal::from(1200),
            period_count,
            period_unit: period_unit.to_string(),
            period_value,
            payment_method: "cash".to_string(),
            remark: None,
            source_credit_card_id: None,
        }
    }

    #[test]
    fn due_dates_follow_period_count_and_step() {
        let input = row(
            NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
            None,
            None,
            3,
            "month",
            1,
        );

        let due_dates = build_due_dates(&input);
        assert_eq!(due_dates.len(), 3);
        assert_eq!(
            due_dates,
            vec![
                NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
                NaiveDate::from_ymd_opt(2026, 2, 10).expect("valid"),
                NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid"),
            ]
        );
    }

    #[test]
    fn due_dates_stop_at_end_date() {
        let input = row(
            NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
            Some(NaiveDate::from_ymd_opt(2026, 2, 15).expect("valid")),
            None,
            6,
            "month",
            1,
        );

        let due_dates = build_due_dates(&input);
        assert_eq!(
            due_dates,
            vec![
                NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
                NaiveDate::from_ymd_opt(2026, 2, 10).expect("valid"),
            ]
        );
    }

    #[test]
    fn due_dates_stop_at_repay_deadline() {
        let input = row(
            NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
            None,
            Some(NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid")),
            12,
            "month",
            1,
        );

        let due_dates = build_due_dates(&input);
        assert_eq!(
            due_dates,
            vec![
                NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
                NaiveDate::from_ymd_opt(2026, 2, 10).expect("valid"),
                NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid"),
            ]
        );
    }
}
