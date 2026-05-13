use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::brand::{BrandListQuery, CreateBrandRequest, UpdateBrandRequest},
    error::app_error::AppError,
    model::brand::Brand,
};

fn map_brand(row: sqlx::mysql::MySqlRow) -> Result<Brand, sqlx::Error> {
    Ok(Brand {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        brand_name: row.try_get("brand_name")?,
        score: row.try_get("score")?,
        board_type: row.try_get("board_type")?,
        review: row.try_get("review")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &BrandListQuery,
) -> Result<(Vec<Brand>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let category_id = query.category_id.map(parse_u64_id).transpose()?;
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM brands
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR board_type = ?)
           AND (? IS NULL OR category_id = ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.board_type)
    .bind(&query.board_type)
    .bind(category_id)
    .bind(category_id)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, category_id, category_name, brand_name, score, board_type, review, remark, created_at, updated_at
         FROM brands
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR board_type = ?)
           AND (? IS NULL OR category_id = ?)
         ORDER BY score DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.board_type)
    .bind(&query.board_type)
    .bind(category_id)
    .bind(category_id)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(map_brand)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Brand, AppError> {
    let row = sqlx::query(
        "SELECT id, user_id, category_id, category_name, brand_name, score, board_type, review, remark, created_at, updated_at
         FROM brands
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_brand).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateBrandRequest,
    category_name: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO brands (user_id, category_id, category_name, brand_name, score, board_type, review, remark)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.brand_name)
    .bind(payload.score)
    .bind(&payload.board_type)
    .bind(&payload.review)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateBrandRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE brands
         SET category_id = ?, category_name = ?, brand_name = ?, score = ?, board_type = ?, review = ?, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.brand_name)
    .bind(payload.score)
    .bind(&payload.board_type)
    .bind(&payload.review)
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
        "UPDATE brands SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
