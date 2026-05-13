use sqlx::{MySqlPool, Row};

use crate::error::app_error::AppError;

#[derive(Debug, Clone)]
pub struct AuditUserRow {
    pub id: u64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub enabled: bool,
}

pub async fn ping_legacy_database(pool: &MySqlPool) -> Result<bool, AppError> {
    let row = sqlx::query("SELECT 1 AS ok").fetch_one(pool).await?;
    let value: i32 = row.try_get("ok")?;
    Ok(value == 1)
}

pub async fn count_rows(pool: &MySqlPool, table: &str) -> Result<u64, AppError> {
    let sql = format!("SELECT COUNT(*) AS total FROM {table}");
    let row = sqlx::query(&sql).fetch_one(pool).await?;
    let total: i64 = row.try_get("total")?;
    Ok(total.max(0) as u64)
}

pub async fn count_permission_snapshots(pool: &MySqlPool) -> Result<u64, AppError> {
    let row = sqlx::query("SELECT COUNT(*) AS total FROM intel_items WHERE source = 'pfm_auth'")
        .fetch_one(pool)
        .await?;
    let total: i64 = row.try_get("total")?;
    Ok(total.max(0) as u64)
}

pub async fn list_permission_snapshot_user_ids(pool: &MySqlPool) -> Result<Vec<u64>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT user_id
        FROM intel_items
        WHERE source = 'pfm_auth' AND deleted_at IS NULL
        ORDER BY user_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| row.try_get::<u64, _>("user_id"))
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::from)
}

pub async fn list_users(pool: &MySqlPool) -> Result<Vec<AuditUserRow>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, username, display_name, role, enabled
        FROM users
        WHERE deleted_at IS NULL
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AuditUserRow {
                id: row.try_get::<u64, _>("id")?,
                username: row.try_get::<String, _>("username")?,
                display_name: row.try_get::<String, _>("display_name")?,
                role: row.try_get::<String, _>("role")?,
                enabled: row.try_get::<bool, _>("enabled")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::from)
}
