use chrono::NaiveDateTime;
use sqlx::Row;
use std::collections::HashSet;

use crate::{
    common::id::parse_u64_id,
    dto::bill_tag::{BillTagListQuery, CreateBillTagRequest, UpdateBillTagRequest},
    error::app_error::AppError,
    model::bill_tag::BillTag,
};

fn map_row(row: sqlx::mysql::MySqlRow) -> Result<BillTag, sqlx::Error> {
    Ok(BillTag {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        name: row.try_get("name")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &BillTagListQuery,
) -> Result<(Vec<BillTag>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));

    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM bill_tags
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR name LIKE ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
                "SELECT id, user_id, name, created_at, updated_at
         FROM bill_tags
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR name LIKE ?)
         ORDER BY id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&keyword)
    .bind(&keyword)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_row)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_name(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    name: &str,
) -> Result<Option<BillTag>, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, name, created_at, updated_at
         FROM bill_tags
         WHERE user_id = ? AND name = ? AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    row.map(map_row).transpose().map_err(AppError::from)
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<BillTag, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, name, created_at, updated_at
         FROM bill_tags
         WHERE id = ? AND deleted_at IS NULL
         LIMIT 1",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_row).transpose()?.ok_or(AppError::NotFound)
}

pub async fn count_by_user(pool: &sqlx::MySqlPool, user_id: u64) -> Result<u64, AppError> {
    let row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM bill_tags
         WHERE user_id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(user_id)?)
    .fetch_one(pool)
    .await?;
    Ok(row.try_get::<i64, _>("total")?.max(0) as u64)
}

pub async fn list_legacy_label_names(pool: &sqlx::MySqlPool) -> Result<Vec<String>, AppError> {
    let rows = sqlx::query(
        "SELECT name
         FROM account_label
         WHERE name IS NOT NULL
         ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for row in rows {
        let Some(name) = row.try_get::<Option<String>, _>("name")? else {
            continue;
        };
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            result.push(trimmed.to_string());
        }
    }
    Ok(result)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateBillTagRequest,
) -> Result<u64, AppError> {
    let result = sqlx::query("INSERT INTO bill_tags (user_id, name) VALUES (?, ?)")
        .bind(parse_u64_id(payload.user_id)?)
        .bind(payload.name.trim())
        .execute(pool)
        .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateBillTagRequest,
) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE bill_tags SET name = ? WHERE id = ? AND deleted_at IS NULL")
        .bind(payload.name.trim())
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
        "UPDATE bill_tags SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Returns up to 20 most frequently used tag names for a user, ordered by usage count DESC.
pub async fn top_tags(pool: &sqlx::MySqlPool, user_id: u64) -> Result<Vec<BillTag>, AppError> {
    let user_id_i64 = parse_u64_id(user_id)?;
    let rows = sqlx::query(
        "SELECT bt.id, bt.user_id, bt.name, bt.created_at, bt.updated_at
         FROM bill_tags bt
         LEFT JOIN (
             SELECT jt.tag_name, COUNT(*) AS usage_count
             FROM bills b
             JOIN JSON_TABLE(b.tags, '$[*]' COLUMNS (tag_name VARCHAR(64) PATH '$')) AS jt
             WHERE b.user_id = ? AND b.deleted_at IS NULL
             GROUP BY jt.tag_name
         ) usage ON usage.tag_name = bt.name
         WHERE bt.user_id = ? AND bt.deleted_at IS NULL
         ORDER BY COALESCE(usage.usage_count, 0) DESC, bt.id DESC
         LIMIT 20",
    )
    .bind(user_id_i64)
    .bind(user_id_i64)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(map_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}
