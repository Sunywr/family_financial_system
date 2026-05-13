use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::intel::{CreateIntelRequest, IntelListQuery, UpdateIntelRequest},
    error::app_error::AppError,
    model::intel_item::IntelItem,
};

fn map_intel(row: sqlx::mysql::MySqlRow) -> Result<IntelItem, sqlx::Error> {
    Ok(IntelItem {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        title: row.try_get("title")?,
        source: row.try_get("source")?,
        item_date: row.try_get::<NaiveDate, _>("item_date")?,
        status: row.try_get("status")?,
        tags: row.try_get("tags")?,
        summary: row.try_get("summary")?,
        content: row.try_get("content")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &IntelListQuery,
) -> Result<(Vec<IntelItem>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));

    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM intel_items
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR title LIKE ? OR summary LIKE ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, title, source, item_date, status, tags, summary, content, created_at, updated_at
         FROM intel_items
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR title LIKE ? OR summary LIKE ?)
         ORDER BY item_date DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_intel)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<IntelItem, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, title, source, item_date, status, tags, summary, content, created_at, updated_at
         FROM intel_items
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_intel).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(pool: &sqlx::MySqlPool, payload: &CreateIntelRequest) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO intel_items (user_id, title, source, item_date, status, tags, summary, content)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(&payload.title)
    .bind(&payload.source)
    .bind(payload.item_date)
    .bind(&payload.status)
    .bind(&payload.tags)
    .bind(&payload.summary)
    .bind(&payload.content)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateIntelRequest,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE intel_items
         SET title = ?, source = ?, item_date = ?, status = ?, tags = ?, summary = ?, content = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.title)
    .bind(&payload.source)
    .bind(payload.item_date)
    .bind(&payload.status)
    .bind(&payload.tags)
    .bind(&payload.summary)
    .bind(&payload.content)
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
        "UPDATE intel_items SET deleted_at = CURRENT_TIMESTAMP
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
