use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::presale::{CreatePresaleRequest, PresaleListQuery, UpdatePresaleRequest},
    error::app_error::AppError,
    model::presale::Presale,
};

fn map_presale(row: sqlx::mysql::MySqlRow) -> Result<Presale, sqlx::Error> {
    Ok(Presale {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        source_bill_id: row.try_get("source_bill_id")?,
        deposit_date: row.try_get::<NaiveDate, _>("deposit_date")?,
        final_payment_date: row.try_get("final_payment_date")?,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        deposit_amount: row.try_get("deposit_amount")?,
        final_payment_amount: row.try_get("final_payment_amount")?,
        status: row.try_get("status")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &PresaleListQuery,
) -> Result<(Vec<Presale>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM presales
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
        "SELECT id, user_id, source_bill_id, deposit_date, final_payment_date, category_id,
                category_name, CAST(deposit_amount AS CHAR) AS deposit_amount,
                CAST(final_payment_amount AS CHAR) AS final_payment_amount, status, remark, created_at, updated_at
         FROM presales
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR category_name LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)
         ORDER BY deposit_date DESC, id DESC
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
            .map(map_presale)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Presale, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, source_bill_id, deposit_date, final_payment_date, category_id,
                category_name, CAST(deposit_amount AS CHAR) AS deposit_amount,
                CAST(final_payment_amount AS CHAR) AS final_payment_amount, status, remark, created_at, updated_at
         FROM presales
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_presale).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreatePresaleRequest,
    category_name: &str,
    source_bill_id: Option<u64>,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO presales (
            user_id, source_bill_id, deposit_date, final_payment_date, category_id, category_name,
            deposit_amount, final_payment_amount, remark
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(source_bill_id.map(parse_u64_id).transpose()?)
    .bind(payload.deposit_date)
    .bind(payload.final_payment_date)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.deposit_amount)
    .bind(
        payload
            .final_payment_amount
            .clone()
            .unwrap_or_else(|| "0.00".to_string()),
    )
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdatePresaleRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE presales
         SET deposit_date = ?, final_payment_date = ?, category_id = ?, category_name = ?,
             deposit_amount = ?, final_payment_amount = ?, status = ?, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(payload.deposit_date)
    .bind(payload.final_payment_date)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.deposit_amount)
    .bind(&payload.final_payment_amount)
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
        "UPDATE presales SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
