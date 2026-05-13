use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::config_item::{ConfigItemListQuery, CreateConfigItemRequest, UpdateConfigItemRequest},
    error::app_error::AppError,
    model::config_item::ConfigItem,
};

fn map_config_item(row: sqlx::mysql::MySqlRow) -> Result<ConfigItem, sqlx::Error> {
    Ok(ConfigItem {
        id: row.try_get::<u64, _>("id")?,
        config_type: row.try_get("config_type")?,
        name: row.try_get("name")?,
        display_name: row.try_get("display_name")?,
        is_builtin: row.try_get::<bool, _>("is_builtin")?,
        enabled: row.try_get::<bool, _>("enabled")?,
        sort_order: row.try_get("sort_order")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &ConfigItemListQuery,
) -> Result<(Vec<ConfigItem>, u64), AppError> {
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM config_items
         WHERE deleted_at IS NULL
           AND (? IS NULL OR config_type = ?)
           AND (? IS NULL OR enabled = ?)
           AND (? IS NULL OR name LIKE ? OR display_name LIKE ?)",
    )
    .bind(&query.config_type)
    .bind(&query.config_type)
    .bind(query.enabled)
    .bind(query.enabled)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, config_type, name, display_name, is_builtin, enabled, sort_order, created_at, updated_at
         FROM config_items
         WHERE deleted_at IS NULL
           AND (? IS NULL OR config_type = ?)
           AND (? IS NULL OR enabled = ?)
           AND (? IS NULL OR name LIKE ? OR display_name LIKE ?)
         ORDER BY config_type ASC, sort_order ASC, id ASC
         LIMIT ? OFFSET ?",
    )
    .bind(&query.config_type)
    .bind(&query.config_type)
    .bind(query.enabled)
    .bind(query.enabled)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    let list = rows
        .into_iter()
        .map(map_config_item)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((list, total))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<ConfigItem, AppError> {
    let id = parse_u64_id(id)?;
    let row = sqlx::query(
        "SELECT id, config_type, name, display_name, is_builtin, enabled, sort_order, created_at, updated_at
         FROM config_items
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    row.map(map_config_item)
        .transpose()?
        .ok_or(AppError::NotFound)
}

pub async fn find_by_type_and_name(
    pool: &sqlx::MySqlPool,
    config_type: &str,
    name: &str,
) -> Result<Option<ConfigItem>, AppError> {
    let row = sqlx::query(
        "SELECT id, config_type, name, display_name, is_builtin, enabled, sort_order, created_at, updated_at
         FROM config_items
         WHERE config_type = ? AND name = ? AND deleted_at IS NULL",
    )
    .bind(config_type)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    row.map(map_config_item).transpose().map_err(AppError::from)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateConfigItemRequest,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO config_items (config_type, name, display_name, is_builtin, enabled, sort_order)
         VALUES (?, ?, ?, 0, ?, ?)",
    )
    .bind(&payload.config_type)
    .bind(&payload.name)
    .bind(&payload.display_name)
    .bind(payload.enabled.unwrap_or(true))
    .bind(payload.sort_order.unwrap_or(0))
    .execute(pool)
    .await
    .map_err(|error| {
        if matches!(&error, sqlx::Error::Database(db_err) if db_err.is_unique_violation()) {
            AppError::Conflict("config item already exists".to_string())
        } else {
            AppError::from(error)
        }
    })?;

    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateConfigItemRequest,
) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let result = sqlx::query(
        "UPDATE config_items
         SET display_name = ?, enabled = ?, sort_order = ?
         WHERE id = ? AND deleted_at IS NULL AND is_builtin = 0",
    )
    .bind(&payload.display_name)
    .bind(payload.enabled)
    .bind(payload.sort_order)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest(
            "builtin config items cannot be modified or item was not found".to_string(),
        ));
    }

    Ok(())
}

pub async fn soft_delete(pool: &sqlx::MySqlPool, id: u64) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let result = sqlx::query(
        "UPDATE config_items
         SET deleted_at = CURRENT_TIMESTAMP
         WHERE id = ? AND deleted_at IS NULL AND is_builtin = 0",
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest(
            "builtin config items cannot be deleted or item was not found".to_string(),
        ));
    }

    Ok(())
}

pub async fn seed_builtin_items(
    pool: &sqlx::MySqlPool,
    items: &[(&str, &str, &str, i32)],
) -> Result<(), AppError> {
    for (config_type, name, display_name, sort_order) in items {
        sqlx::query(
            "INSERT INTO config_items (config_type, name, display_name, is_builtin, enabled, sort_order)
             VALUES (?, ?, ?, 1, 1, ?)
             ON DUPLICATE KEY UPDATE display_name = VALUES(display_name), sort_order = VALUES(sort_order)",
        )
        .bind(config_type)
        .bind(name)
        .bind(display_name)
        .bind(sort_order)
        .execute(pool)
        .await?;
    }

    Ok(())
}
