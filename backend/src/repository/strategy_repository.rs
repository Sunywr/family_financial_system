use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::strategy::{CreateStrategyRequest, StrategyListQuery, UpdateStrategyRequest},
    error::app_error::AppError,
    model::strategy_config::StrategyConfig,
};

fn map_strategy(row: sqlx::mysql::MySqlRow) -> Result<StrategyConfig, sqlx::Error> {
    Ok(StrategyConfig {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        investment_type: row.try_get("investment_type")?,
        target_code: row.try_get("target_code")?,
        strategy_name: row.try_get("strategy_name")?,
        enabled: row.try_get("enabled")?,
        risk_level: row.try_get("risk_level")?,
        preferred_min_score: row.try_get("preferred_min_score")?,
        cooldown_days: row.try_get("cooldown_days")?,
        take_profit_rate: row.try_get("take_profit_rate")?,
        stop_loss_rate: row.try_get("stop_loss_rate")?,
        notes: row.try_get("notes")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &StrategyListQuery,
) -> Result<(Vec<StrategyConfig>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM strategy_configs
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR investment_type = ?)
           AND (? IS NULL OR enabled = ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.investment_type)
    .bind(&query.investment_type)
    .bind(query.enabled)
    .bind(query.enabled)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, investment_type, target_code, strategy_name, enabled, risk_level,
                preferred_min_score, cooldown_days, CAST(take_profit_rate AS CHAR) AS take_profit_rate,
                CAST(stop_loss_rate AS CHAR) AS stop_loss_rate, notes, created_at, updated_at
         FROM strategy_configs
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR investment_type = ?)
           AND (? IS NULL OR enabled = ?)
         ORDER BY updated_at DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.investment_type)
    .bind(&query.investment_type)
    .bind(query.enabled)
    .bind(query.enabled)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_strategy)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<StrategyConfig, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, investment_type, target_code, strategy_name, enabled, risk_level,
                preferred_min_score, cooldown_days, CAST(take_profit_rate AS CHAR) AS take_profit_rate,
                CAST(stop_loss_rate AS CHAR) AS stop_loss_rate, notes, created_at, updated_at
         FROM strategy_configs
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_strategy).transpose()?.ok_or(AppError::NotFound)
}

pub async fn list_enabled_by_user_type(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    investment_type: &str,
) -> Result<Vec<StrategyConfig>, AppError> {
    let rows = sqlx::query(
        "SELECT id, user_id, investment_type, target_code, strategy_name, enabled, risk_level,
                preferred_min_score, cooldown_days, CAST(take_profit_rate AS CHAR) AS take_profit_rate,
                CAST(stop_loss_rate AS CHAR) AS stop_loss_rate, notes, created_at, updated_at
         FROM strategy_configs
         WHERE user_id = ? AND investment_type = ? AND enabled = 1 AND deleted_at IS NULL
         ORDER BY target_code IS NULL DESC, updated_at DESC",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(investment_type)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(map_strategy)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateStrategyRequest,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO strategy_configs (
            user_id, investment_type, target_code, strategy_name, enabled, risk_level,
            preferred_min_score, cooldown_days, take_profit_rate, stop_loss_rate, notes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(&payload.investment_type)
    .bind(&payload.target_code)
    .bind(&payload.strategy_name)
    .bind(payload.enabled)
    .bind(&payload.risk_level)
    .bind(payload.preferred_min_score)
    .bind(payload.cooldown_days)
    .bind(&payload.take_profit_rate)
    .bind(&payload.stop_loss_rate)
    .bind(&payload.notes)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateStrategyRequest,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE strategy_configs
         SET investment_type = ?, target_code = ?, strategy_name = ?, enabled = ?, risk_level = ?,
             preferred_min_score = ?, cooldown_days = ?, take_profit_rate = ?, stop_loss_rate = ?, notes = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.investment_type)
    .bind(&payload.target_code)
    .bind(&payload.strategy_name)
    .bind(payload.enabled)
    .bind(&payload.risk_level)
    .bind(payload.preferred_min_score)
    .bind(payload.cooldown_days)
    .bind(&payload.take_profit_rate)
    .bind(&payload.stop_loss_rate)
    .bind(&payload.notes)
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
        "UPDATE strategy_configs
         SET deleted_at = CURRENT_TIMESTAMP
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
