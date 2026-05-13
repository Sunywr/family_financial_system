use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::balance_calibration::{
        BalanceCalibrationListQuery, CreateBalanceCalibrationRequest,
        UpdateBalanceCalibrationRequest,
    },
    error::app_error::AppError,
    model::balance_calibration::BalanceCalibration,
};

fn map_item(row: sqlx::mysql::MySqlRow) -> Result<BalanceCalibration, sqlx::Error> {
    Ok(BalanceCalibration {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        calibration_date: row.try_get::<NaiveDate, _>("calibration_date")?,
        cash_balance: row.try_get("cash_balance")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &BalanceCalibrationListQuery,
) -> Result<(Vec<BalanceCalibration>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM balance_calibrations
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, calibration_date, CAST(cash_balance AS CHAR) AS cash_balance, remark, created_at, updated_at
         FROM balance_calibrations
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
         ORDER BY calibration_date DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_item)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<BalanceCalibration, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, calibration_date, CAST(cash_balance AS CHAR) AS cash_balance, remark, created_at, updated_at
         FROM balance_calibrations
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_item).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateBalanceCalibrationRequest,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO balance_calibrations (user_id, calibration_date, cash_balance, remark)
         VALUES (?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(payload.calibration_date)
    .bind(&payload.cash_balance)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateBalanceCalibrationRequest,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE balance_calibrations
         SET calibration_date = ?, cash_balance = ?, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(payload.calibration_date)
    .bind(&payload.cash_balance)
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
        "UPDATE balance_calibrations SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn latest_before_or_on(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    end_date: NaiveDate,
) -> Result<Option<BalanceCalibration>, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, calibration_date, CAST(cash_balance AS CHAR) AS cash_balance, remark, created_at, updated_at
         FROM balance_calibrations
         WHERE user_id = ? AND calibration_date <= ? AND deleted_at IS NULL
         ORDER BY calibration_date DESC, id DESC
         LIMIT 1",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(end_date)
    .fetch_optional(pool)
    .await?;
    row.map(map_item).transpose().map_err(AppError::from)
}
