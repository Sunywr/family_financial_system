use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::budget::{BudgetListQuery, CreateBudgetRequest, UpdateBudgetRequest},
    error::app_error::AppError,
    model::budget::Budget,
};

fn map_budget(row: sqlx::mysql::MySqlRow) -> Result<Budget, sqlx::Error> {
    Ok(Budget {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        budget_month: row.try_get::<NaiveDate, _>("budget_month")?,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        planned_amount: row.try_get("planned_amount")?,
        actual_amount: row.try_get("actual_amount")?,
        manual_adjusted: row.try_get("manual_adjusted")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &BudgetListQuery,
) -> Result<(Vec<Budget>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM budgets
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR budget_month = ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(query.budget_month)
    .bind(query.budget_month)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT b.id, b.user_id, b.budget_month, b.category_id, b.category_name, CAST(b.planned_amount AS CHAR) AS planned_amount,
                CAST(COALESCE((
                    SELECT SUM(amount)
                    FROM bills bl
                    WHERE bl.user_id = b.user_id
                      AND bl.category_id = b.category_id
                      AND bl.bill_type IN ('expense','open_position','add_position')
                      AND DATE_FORMAT(bl.account_date, '%Y-%m-01') = b.budget_month
                      AND bl.deleted_at IS NULL
                ), 0) AS CHAR) AS actual_amount,
                b.manual_adjusted, b.remark, b.created_at, b.updated_at
         FROM budgets b
         WHERE b.deleted_at IS NULL
           AND (? IS NULL OR b.user_id = ?)
           AND (? IS NULL OR b.budget_month = ?)
         ORDER BY b.budget_month DESC, b.category_name ASC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(query.budget_month)
    .bind(query.budget_month)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_budget)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Budget, AppError> {
    let row = sqlx::query(
        "SELECT b.id, b.user_id, b.budget_month, b.category_id, b.category_name, CAST(b.planned_amount AS CHAR) AS planned_amount,
                CAST(COALESCE((
                    SELECT SUM(amount)
                    FROM bills bl
                    WHERE bl.user_id = b.user_id
                      AND bl.category_id = b.category_id
                      AND bl.bill_type IN ('expense','open_position','add_position')
                      AND DATE_FORMAT(bl.account_date, '%Y-%m-01') = b.budget_month
                      AND bl.deleted_at IS NULL
                ), 0) AS CHAR) AS actual_amount,
                b.manual_adjusted, b.remark, b.created_at, b.updated_at
         FROM budgets b
         WHERE b.id = ? AND b.deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_budget).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateBudgetRequest,
    category_name: &str,
    manual_adjusted: bool,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO budgets (user_id, budget_month, category_id, category_name, planned_amount, manual_adjusted, remark)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(payload.budget_month)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.planned_amount)
    .bind(manual_adjusted)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateBudgetRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE budgets
         SET budget_month = ?, category_id = ?, category_name = ?, planned_amount = ?, manual_adjusted = 1, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(payload.budget_month)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.planned_amount)
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
        "UPDATE budgets SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn upsert_generated(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    budget_month: NaiveDate,
    category_id: u64,
    category_name: &str,
    planned_amount: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO budgets (user_id, budget_month, category_id, category_name, planned_amount, manual_adjusted)
         VALUES (?, ?, ?, ?, ?, 0)
         ON DUPLICATE KEY UPDATE
           planned_amount = IF(manual_adjusted = 1, planned_amount, VALUES(planned_amount)),
           category_name = VALUES(category_name)",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(budget_month)
    .bind(parse_u64_id(category_id)?)
    .bind(category_name)
    .bind(planned_amount)
    .execute(pool)
    .await?;
    Ok(())
}
