use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::credit_card::{CreateCreditCardRequest, CreditCardListQuery, UpdateCreditCardRequest},
    error::app_error::AppError,
    model::credit_card::CreditCard,
};

fn map_credit_card(row: sqlx::mysql::MySqlRow) -> Result<CreditCard, sqlx::Error> {
    Ok(CreditCard {
        id: row.try_get::<u64, _>("id")?,
        user_id: row.try_get::<u64, _>("user_id")?,
        name: row.try_get("name")?,
        billing_day: row.try_get("billing_day")?,
        repayment_day: row.try_get("repayment_day")?,
        credit_limit: row.try_get::<String, _>("credit_limit")?,
        enabled: row.try_get::<bool, _>("enabled")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &CreditCardListQuery,
) -> Result<(Vec<CreditCard>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM credit_cards
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR enabled = ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(query.enabled)
    .bind(query.enabled)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, name, billing_day, repayment_day, CAST(credit_limit AS CHAR) AS credit_limit, enabled, created_at, updated_at
         FROM credit_cards
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR enabled = ?)
         ORDER BY id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(query.enabled)
    .bind(query.enabled)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    let list = rows
        .into_iter()
        .map(map_credit_card)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((list, total))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<CreditCard, AppError> {
    let id = parse_u64_id(id)?;
    let row = sqlx::query(
        "SELECT id, user_id, name, billing_day, repayment_day, CAST(credit_limit AS CHAR) AS credit_limit, enabled, created_at, updated_at
         FROM credit_cards
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    row.map(map_credit_card)
        .transpose()?
        .ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateCreditCardRequest,
) -> Result<u64, AppError> {
    let user_id = parse_u64_id(payload.user_id)?;
    let result = sqlx::query(
        "INSERT INTO credit_cards (user_id, name, billing_day, repayment_day, credit_limit, enabled)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(payload.billing_day)
    .bind(payload.repayment_day)
    .bind(&payload.credit_limit)
    .bind(payload.enabled.unwrap_or(true))
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateCreditCardRequest,
) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let result = sqlx::query(
        "UPDATE credit_cards
         SET name = ?, billing_day = ?, repayment_day = ?, credit_limit = ?, enabled = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.name)
    .bind(payload.billing_day)
    .bind(payload.repayment_day)
    .bind(&payload.credit_limit)
    .bind(payload.enabled)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn soft_delete(pool: &sqlx::MySqlPool, id: u64) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let result = sqlx::query(
        "UPDATE credit_cards
         SET deleted_at = CURRENT_TIMESTAMP
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
