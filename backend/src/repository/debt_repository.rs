use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::debt::{CreateDebtRequest, DebtListQuery, UpdateDebtRequest},
    error::app_error::AppError,
    model::debt::Debt,
};

fn map_debt(row: sqlx::mysql::MySqlRow) -> Result<Debt, sqlx::Error> {
    Ok(Debt {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        source_bill_id: row.try_get("source_bill_id")?,
        start_date: row.try_get::<NaiveDate, _>("start_date")?,
        end_date: row.try_get("end_date")?,
        repay_deadline: row.try_get("repay_deadline")?,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        amount: row.try_get("amount")?,
        period_count: row.try_get("period_count")?,
        period_unit: row.try_get("period_unit")?,
        period_value: row.try_get("period_value")?,
        payment_method: row.try_get("payment_method")?,
        status: row.try_get("status")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &DebtListQuery,
) -> Result<(Vec<Debt>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM debts
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR category_name LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, source_bill_id, start_date, end_date, repay_deadline, category_id,
                category_name, CAST(amount AS CHAR) AS amount, period_count, period_unit, period_value, payment_method,
                status, remark, created_at, updated_at
         FROM debts
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR category_name LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)
         ORDER BY start_date DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_debt)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Debt, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, source_bill_id, start_date, end_date, repay_deadline, category_id,
                category_name, CAST(amount AS CHAR) AS amount, period_count, period_unit, period_value, payment_method,
                status, remark, created_at, updated_at
         FROM debts
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_debt).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateDebtRequest,
    category_name: &str,
    source_bill_id: Option<u64>,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO debts (
            user_id, source_bill_id, start_date, end_date, repay_deadline, category_id, category_name,
            amount, period_count, period_unit, period_value, payment_method, remark
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(source_bill_id.map(parse_u64_id).transpose()?)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.repay_deadline)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(payload.period_count)
    .bind(&payload.period_unit)
    .bind(payload.period_value)
    .bind(&payload.payment_method)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateDebtRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE debts
         SET start_date = ?, end_date = ?, repay_deadline = ?, category_id = ?, category_name = ?,
             amount = ?, period_count = ?, period_unit = ?, period_value = ?, payment_method = ?,
             status = ?, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.repay_deadline)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(payload.period_count)
    .bind(&payload.period_unit)
    .bind(payload.period_value)
    .bind(&payload.payment_method)
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
        "UPDATE debts SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
