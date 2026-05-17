use chrono::NaiveDate;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::auto_invest_plan::{
        AutoInvestPlanListQuery, CreateAutoInvestPlanRequest, UpdateAutoInvestPlanRequest,
    },
    error::app_error::AppError,
    model::auto_invest_plan::AutoInvestPlan,
};

const SELECT_COLS: &str = "id, user_id, investment_id, investment_name, name, \
    category_id, category_name, amount, cycle_months, start_date, end_date, \
    last_generated_date, status, remark, created_at, updated_at";

fn map_plan(row: sqlx::mysql::MySqlRow) -> Result<AutoInvestPlan, sqlx::Error> {
    Ok(AutoInvestPlan {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        investment_id: row.try_get("investment_id")?,
        investment_name: row.try_get("investment_name")?,
        name: row.try_get("name")?,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        amount: row.try_get("amount")?,
        cycle_months: row.try_get("cycle_months")?,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date")?,
        last_generated_date: row.try_get("last_generated_date")?,
        status: row.try_get("status")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &AutoInvestPlanListQuery,
) -> Result<(Vec<AutoInvestPlan>, u64), AppError> {
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM auto_invest_plans
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)",
    )
    .bind(query.user_id.map(|v| v as i64))
    .bind(query.user_id.map(|v| v as i64))
    .bind(query.status.as_deref())
    .bind(query.status.as_deref())
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let sql = format!(
        "SELECT {SELECT_COLS}
         FROM auto_invest_plans
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
         ORDER BY id DESC
         LIMIT ? OFFSET ?"
    );
    let rows = sqlx::query(&sql)
        .bind(query.user_id.map(|v| v as i64))
        .bind(query.user_id.map(|v| v as i64))
        .bind(query.status.as_deref())
        .bind(query.status.as_deref())
        .bind(query.pagination.page_size)
        .bind(query.pagination.offset())
        .fetch_all(pool)
        .await?;

    Ok((
        rows.into_iter()
            .map(map_plan)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<AutoInvestPlan, AppError> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM auto_invest_plans WHERE id = ? AND deleted_at IS NULL"
    );
    let row = sqlx::query(&sql)
        .bind(parse_u64_id(id)?)
        .fetch_optional(pool)
        .await?;
    row.map(map_plan).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateAutoInvestPlanRequest,
    investment_name: &str,
    category_name: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO auto_invest_plans
            (user_id, investment_id, investment_name, name, category_id, category_name,
             amount, cycle_months, start_date, end_date, status, remark)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(parse_u64_id(payload.investment_id)?)
    .bind(investment_name)
    .bind(&payload.name)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(payload.cycle_months)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateAutoInvestPlanRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE auto_invest_plans
         SET name = ?, category_id = ?, category_name = ?, amount = ?,
             cycle_months = ?, start_date = ?, end_date = ?, status = ?, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.name)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(payload.cycle_months)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(&payload.status)
    .bind(&payload.remark)
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn soft_delete(pool: &sqlx::MySqlPool, id: u64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE auto_invest_plans
         SET deleted_at = CURRENT_TIMESTAMP
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn list_active_for_scheduler(
    pool: &sqlx::MySqlPool,
    today: NaiveDate,
) -> Result<Vec<AutoInvestPlan>, AppError> {
    let sql = format!(
        "SELECT {SELECT_COLS}
         FROM auto_invest_plans
         WHERE deleted_at IS NULL
           AND status = 'active'
           AND start_date <= ?
         ORDER BY id ASC"
    );
    let rows = sqlx::query(&sql)
        .bind(today)
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(map_plan)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

pub async fn update_last_generated_date(
    pool: &sqlx::MySqlPool,
    id: u64,
    date: NaiveDate,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE auto_invest_plans SET last_generated_date = ? WHERE id = ?",
    )
    .bind(date)
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    Ok(())
}
