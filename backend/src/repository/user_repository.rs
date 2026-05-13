use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::user::{CreateUserRequest, UpdateUserRequest, UserListQuery},
    error::app_error::AppError,
    model::user::User,
};

#[derive(Debug, Clone)]
pub struct UserCredential {
    pub id: u64,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub role: String,
    pub enabled: bool,
}

fn map_user(row: sqlx::mysql::MySqlRow) -> Result<User, sqlx::Error> {
    Ok(User {
        id: row.try_get::<u64, _>("id")?,
        username: row.try_get("username")?,
        display_name: row.try_get("display_name")?,
        role: row.try_get("role")?,
        enabled: row.try_get::<bool, _>("enabled")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &UserListQuery,
) -> Result<(Vec<User>, u64), AppError> {
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM users
         WHERE deleted_at IS NULL
           AND (? IS NULL OR enabled = ?)
           AND (? IS NULL OR username LIKE ? OR display_name LIKE ?)",
    )
    .bind(query.enabled)
    .bind(query.enabled)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, username, display_name, role, enabled, created_at, updated_at
         FROM users
         WHERE deleted_at IS NULL
           AND (? IS NULL OR enabled = ?)
           AND (? IS NULL OR username LIKE ? OR display_name LIKE ?)
         ORDER BY id DESC
         LIMIT ? OFFSET ?",
    )
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
        .map(map_user)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((list, total))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<User, AppError> {
    let id = parse_u64_id(id)?;
    let row = sqlx::query(
        "SELECT id, username, display_name, role, enabled, created_at, updated_at
         FROM users
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    row.map(map_user).transpose()?.ok_or(AppError::NotFound)
}

pub async fn find_credential_by_username(
    pool: &sqlx::MySqlPool,
    username: &str,
) -> Result<Option<UserCredential>, AppError> {
    let row = sqlx::query(
        "SELECT id, username, display_name, password_hash, role, enabled
         FROM users
         WHERE username = ? AND deleted_at IS NULL",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        Ok(Some(UserCredential {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            display_name: row.try_get("display_name")?,
            password_hash: row.try_get("password_hash")?,
            role: row.try_get("role")?,
            enabled: row.try_get("enabled")?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn create(pool: &sqlx::MySqlPool, payload: &CreateUserRequest) -> Result<u64, AppError> {
    create_with_password_hash(
        pool,
        &payload.username,
        &payload.display_name,
        &payload.password,
        &payload.role,
        payload.enabled.unwrap_or(true),
    )
    .await
}

pub async fn create_with_password_hash(
    pool: &sqlx::MySqlPool,
    username: &str,
    display_name: &str,
    password_hash: &str,
    role: &str,
    enabled: bool,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO users (username, display_name, password_hash, role, enabled)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(username)
    .bind(display_name)
    .bind(password_hash)
    .bind(role)
    .bind(enabled)
    .execute(pool)
    .await
    .map_err(|error| {
        if matches!(&error, sqlx::Error::Database(db_err) if db_err.is_unique_violation()) {
            AppError::Conflict("username already exists".to_string())
        } else {
            AppError::from(error)
        }
    })?;

    Ok(result.last_insert_id())
}

pub async fn update_password_hash(
    pool: &sqlx::MySqlPool,
    id: u64,
    password_hash: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE users
         SET password_hash = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(password_hash)
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateUserRequest,
) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let result = sqlx::query(
        "UPDATE users
         SET display_name = ?, role = ?, enabled = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.display_name)
    .bind(&payload.role)
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
        "UPDATE users
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

pub async fn list_options(pool: &sqlx::MySqlPool) -> Result<Vec<User>, AppError> {
    let rows = sqlx::query(
        "SELECT id, username, display_name, role, enabled, created_at, updated_at
         FROM users
         WHERE deleted_at IS NULL AND enabled = 1
         ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_user)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

pub async fn seed_default_admin(pool: &sqlx::MySqlPool) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO users (username, display_name, password_hash, role, enabled)
         VALUES ('admin', 'Administrator', ?, 'owner', 1)
         ON DUPLICATE KEY UPDATE display_name = VALUES(display_name), role = VALUES(role), enabled = VALUES(enabled)",
    )
    .bind("sha256$8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918")
    .execute(pool)
    .await?;
    Ok(())
}
