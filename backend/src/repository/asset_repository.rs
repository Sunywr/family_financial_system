use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::asset::{AssetListQuery, CreateAssetRequest, UpdateAssetRequest},
    error::app_error::AppError,
    model::asset::Asset,
};

fn map_asset(row: sqlx::mysql::MySqlRow) -> Result<Asset, sqlx::Error> {
    Ok(Asset {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        source_bill_id: row.try_get("source_bill_id")?,
        name: row.try_get("name")?,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        amount: row.try_get("amount")?,
        remark: row.try_get("remark")?,
        status: row.try_get("status")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &AssetListQuery,
) -> Result<(Vec<Asset>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM assets
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR name LIKE ? OR category_name LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, source_bill_id, name, category_id, category_name, CAST(amount AS CHAR) AS amount, remark,
                status, created_at, updated_at
         FROM assets
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (? IS NULL OR name LIKE ? OR category_name LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)
         ORDER BY created_at DESC, id DESC
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
    .bind(&keyword)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_asset)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Asset, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, source_bill_id, name, category_id, category_name, CAST(amount AS CHAR) AS amount, remark,
                status, created_at, updated_at
         FROM assets
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_asset).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateAssetRequest,
    category_name: &str,
    source_bill_id: Option<u64>,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO assets (user_id, source_bill_id, name, category_id, category_name, amount, remark)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(source_bill_id.map(parse_u64_id).transpose()?)
    .bind(&payload.name)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateAssetRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE assets
         SET name = ?, category_id = ?, category_name = ?, amount = ?, remark = ?, status = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.name)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(&payload.remark)
    .bind(&payload.status)
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
        "UPDATE assets SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
