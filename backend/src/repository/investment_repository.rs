use chrono::NaiveDateTime;
use sqlx::Row;

use crate::{
    common::id::parse_u64_id, dto::investment::InvestmentListQuery, error::app_error::AppError,
    model::investment::Investment,
};

fn map_investment(row: sqlx::mysql::MySqlRow) -> Result<Investment, sqlx::Error> {
    Ok(Investment {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        source_bill_id: row.try_get("source_bill_id")?,
        investment_type: row.try_get("investment_type")?,
        name: row.try_get("name")?,
        code: row.try_get("code")?,
        organization_name: row.try_get("organization_name")?,
        market: row.try_get("market")?,
        total_shares: row.try_get("total_shares")?,
        total_cost: row.try_get("total_cost")?,
        average_cost: row.try_get("average_cost")?,
        current_price: row.try_get("current_price")?,
        market_value: row.try_get("market_value")?,
        realized_profit: row.try_get("realized_profit")?,
        unrealized_profit: row.try_get("unrealized_profit")?,
        total_profit: row.try_get("total_profit")?,
        total_profit_rate: row.try_get("total_profit_rate")?,
        latest_remark: row.try_get("latest_remark")?,
        status: row.try_get("status")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &InvestmentListQuery,
) -> Result<(Vec<Investment>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let status_filter = if query.show_sold.unwrap_or(false) || query.status.is_some() {
        query.status.clone()
    } else {
        Some("holding".to_string())
    };
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
                 FROM investments i
         WHERE deleted_at IS NULL
                     AND (? IS NULL OR i.user_id = ?)
                     AND (? IS NULL OR i.investment_type = ?)
           AND (? IS NULL OR status = ?)
                     AND (
                         ? IS NULL
                         OR i.name LIKE ?
                         OR i.code LIKE ?
                         OR i.organization_name LIKE ?
                         OR CAST(i.id AS CHAR) LIKE ?
                         OR EXISTS (
                                SELECT 1
                                FROM bills b
                                WHERE b.related_investment_id = i.id
                                    AND b.deleted_at IS NULL
                                    AND b.product_name LIKE ?
                         )
                     )",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.investment_type)
    .bind(&query.investment_type)
    .bind(&status_filter)
    .bind(&status_filter)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
                "SELECT i.id, i.user_id, i.source_bill_id, i.investment_type,
                                COALESCE(
                                        NULLIF(
                                                TRIM(
                                                        CASE
                                                                WHEN i.name = i.organization_name THEN (
                                                                        SELECT b.product_name
                                                                        FROM bills b
                                                                        WHERE b.related_investment_id = i.id
                                                                            AND b.deleted_at IS NULL
                                                                            AND b.product_name IS NOT NULL
                                                                            AND TRIM(b.product_name) <> ''
                                                                        ORDER BY b.account_date DESC, b.id DESC
                                                                        LIMIT 1
                                                                )
                                                                ELSE i.name
                                                        END
                                                ),
                                                ''
                                        ),
                                        CASE
                                                WHEN i.investment_type = 'stock' THEN CONCAT('股票 ', i.code)
                                                ELSE i.organization_name
                                        END
                                ) AS name,
                                i.code, i.organization_name, i.market,
                                CAST(i.total_shares AS CHAR) AS total_shares,
                                CAST(i.total_cost AS CHAR) AS total_cost,
                                CAST(i.average_cost AS CHAR) AS average_cost,
                                CAST(i.current_price AS CHAR) AS current_price,
                                CAST(i.market_value AS CHAR) AS market_value,
                                CAST(i.realized_profit AS CHAR) AS realized_profit,
                                CAST(i.unrealized_profit AS CHAR) AS unrealized_profit,
                                CAST(i.total_profit AS CHAR) AS total_profit,
                                CAST(i.total_profit_rate AS CHAR) AS total_profit_rate,
                                                                (
                                                                        SELECT b.remark
                                                                        FROM bills b
                                                                        WHERE b.related_investment_id = i.id
                                                                            AND b.deleted_at IS NULL
                                                                            AND b.remark IS NOT NULL
                                                                            AND TRIM(b.remark) <> ''
                                                                        ORDER BY b.account_date DESC, b.id DESC
                                                                        LIMIT 1
                                                                ) AS latest_remark,
                                i.status, i.created_at, i.updated_at
                 FROM investments i
         WHERE deleted_at IS NULL
                     AND (? IS NULL OR i.user_id = ?)
                     AND (? IS NULL OR i.investment_type = ?)
           AND (? IS NULL OR status = ?)
                     AND (
                         ? IS NULL
                         OR i.name LIKE ?
                         OR i.code LIKE ?
                         OR i.organization_name LIKE ?
                         OR CAST(i.id AS CHAR) LIKE ?
                         OR EXISTS (
                                SELECT 1
                                FROM bills b
                                WHERE b.related_investment_id = i.id
                                    AND b.deleted_at IS NULL
                                    AND b.product_name LIKE ?
                         )
                     )
                 ORDER BY i.updated_at DESC, i.id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.investment_type)
    .bind(&query.investment_type)
    .bind(&status_filter)
    .bind(&status_filter)
    .bind(&keyword)
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
            .map(map_investment)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Investment, AppError> {
    let row = sqlx::query(
        "SELECT i.id, i.user_id, i.source_bill_id, i.investment_type,
                COALESCE(
                    NULLIF(
                        TRIM(
                            CASE
                                WHEN i.name = i.organization_name THEN (
                                    SELECT b.product_name
                                    FROM bills b
                                    WHERE b.related_investment_id = i.id
                                      AND b.deleted_at IS NULL
                                      AND b.product_name IS NOT NULL
                                      AND TRIM(b.product_name) <> ''
                                    ORDER BY b.account_date DESC, b.id DESC
                                    LIMIT 1
                                )
                                ELSE i.name
                            END
                        ),
                        ''
                    ),
                    CASE
                        WHEN i.investment_type = 'stock' THEN CONCAT('股票 ', i.code)
                        ELSE i.organization_name
                    END
                ) AS name,
                i.code, i.organization_name, i.market,
                CAST(i.total_shares AS CHAR) AS total_shares,
                CAST(i.total_cost AS CHAR) AS total_cost,
                CAST(i.average_cost AS CHAR) AS average_cost,
                CAST(i.current_price AS CHAR) AS current_price,
                CAST(i.market_value AS CHAR) AS market_value,
                CAST(i.realized_profit AS CHAR) AS realized_profit,
                CAST(i.unrealized_profit AS CHAR) AS unrealized_profit,
                CAST(i.total_profit AS CHAR) AS total_profit,
                CAST(i.total_profit_rate AS CHAR) AS total_profit_rate,
                                (
                                        SELECT b.remark
                                        FROM bills b
                                        WHERE b.related_investment_id = i.id
                                            AND b.deleted_at IS NULL
                                            AND b.remark IS NOT NULL
                                            AND TRIM(b.remark) <> ''
                                        ORDER BY b.account_date DESC, b.id DESC
                                        LIMIT 1
                                ) AS latest_remark,
                i.status, i.created_at, i.updated_at
         FROM investments i
         WHERE i.id = ? AND i.deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_investment)
        .transpose()?
        .ok_or(AppError::NotFound)
}

pub async fn find_by_user_type_code(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    investment_type: &str,
    code: &str,
) -> Result<Option<Investment>, AppError> {
    let row = sqlx::query(
        "SELECT i.id, i.user_id, i.source_bill_id, i.investment_type,
                COALESCE(
                    NULLIF(
                        TRIM(
                            CASE
                                WHEN i.name = i.organization_name THEN (
                                    SELECT b.product_name
                                    FROM bills b
                                    WHERE b.related_investment_id = i.id
                                      AND b.deleted_at IS NULL
                                      AND b.product_name IS NOT NULL
                                      AND TRIM(b.product_name) <> ''
                                    ORDER BY b.account_date DESC, b.id DESC
                                    LIMIT 1
                                )
                                ELSE i.name
                            END
                        ),
                        ''
                    ),
                    CASE
                        WHEN i.investment_type = 'stock' THEN CONCAT('股票 ', i.code)
                        ELSE i.organization_name
                    END
                ) AS name,
                i.code, i.organization_name, i.market,
                CAST(i.total_shares AS CHAR) AS total_shares,
                CAST(i.total_cost AS CHAR) AS total_cost,
                CAST(i.average_cost AS CHAR) AS average_cost,
                CAST(i.current_price AS CHAR) AS current_price,
                CAST(i.market_value AS CHAR) AS market_value,
                CAST(i.realized_profit AS CHAR) AS realized_profit,
                CAST(i.unrealized_profit AS CHAR) AS unrealized_profit,
                CAST(i.total_profit AS CHAR) AS total_profit,
                CAST(i.total_profit_rate AS CHAR) AS total_profit_rate,
                                (
                                        SELECT b.remark
                                        FROM bills b
                                        WHERE b.related_investment_id = i.id
                                            AND b.deleted_at IS NULL
                                            AND b.remark IS NOT NULL
                                            AND TRIM(b.remark) <> ''
                                        ORDER BY b.account_date DESC, b.id DESC
                                        LIMIT 1
                                ) AS latest_remark,
                i.status, i.created_at, i.updated_at
         FROM investments i
         WHERE i.user_id = ? AND i.investment_type = ? AND i.code = ? AND i.deleted_at IS NULL",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(investment_type)
    .bind(code)
    .fetch_optional(pool)
    .await?;
    row.map(map_investment).transpose().map_err(AppError::from)
}

pub async fn list_for_scoring(
    pool: &sqlx::MySqlPool,
    investment_type: &str,
) -> Result<Vec<Investment>, AppError> {
    let rows = sqlx::query(
        "SELECT i.id, i.user_id, i.source_bill_id, i.investment_type,
                COALESCE(
                    NULLIF(
                        TRIM(
                            CASE
                                WHEN i.name = i.organization_name THEN (
                                    SELECT b.product_name
                                    FROM bills b
                                    WHERE b.related_investment_id = i.id
                                      AND b.deleted_at IS NULL
                                      AND b.product_name IS NOT NULL
                                      AND TRIM(b.product_name) <> ''
                                    ORDER BY b.account_date DESC, b.id DESC
                                    LIMIT 1
                                )
                                ELSE i.name
                            END
                        ),
                        ''
                    ),
                    CASE
                        WHEN i.investment_type = 'stock' THEN CONCAT('股票 ', i.code)
                        ELSE i.organization_name
                    END
                ) AS name,
                i.code, i.organization_name, i.market,
                CAST(i.total_shares AS CHAR) AS total_shares,
                CAST(i.total_cost AS CHAR) AS total_cost,
                CAST(i.average_cost AS CHAR) AS average_cost,
                CAST(i.current_price AS CHAR) AS current_price,
                CAST(i.market_value AS CHAR) AS market_value,
                CAST(i.realized_profit AS CHAR) AS realized_profit,
                CAST(i.unrealized_profit AS CHAR) AS unrealized_profit,
                CAST(i.total_profit AS CHAR) AS total_profit,
                CAST(i.total_profit_rate AS CHAR) AS total_profit_rate,
                                (
                                        SELECT b.remark
                                        FROM bills b
                                        WHERE b.related_investment_id = i.id
                                            AND b.deleted_at IS NULL
                                            AND b.remark IS NOT NULL
                                            AND TRIM(b.remark) <> ''
                                        ORDER BY b.account_date DESC, b.id DESC
                                        LIMIT 1
                                ) AS latest_remark,
                i.status, i.created_at, i.updated_at
         FROM investments i
         WHERE i.investment_type = ? AND i.deleted_at IS NULL
         ORDER BY i.updated_at DESC, i.id DESC",
    )
    .bind(investment_type)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_investment)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    source_bill_id: u64,
    investment_type: &str,
    name: &str,
    code: &str,
    organization_name: &str,
    total_shares: &str,
    total_cost: &str,
    average_cost: &str,
    current_price: &str,
    market_value: &str,
    realized_profit: &str,
    unrealized_profit: &str,
    total_profit: &str,
    total_profit_rate: &str,
    status: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO investments (
            user_id, source_bill_id, investment_type, name, code, organization_name,
            total_shares, total_cost, average_cost, current_price, market_value,
            realized_profit, unrealized_profit, total_profit, total_profit_rate, status
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(parse_u64_id(source_bill_id)?)
    .bind(investment_type)
    .bind(name)
    .bind(code)
    .bind(organization_name)
    .bind(total_shares)
    .bind(total_cost)
    .bind(average_cost)
    .bind(current_price)
    .bind(market_value)
    .bind(realized_profit)
    .bind(unrealized_profit)
    .bind(total_profit)
    .bind(total_profit_rate)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_metrics(
    pool: &sqlx::MySqlPool,
    id: u64,
    name: &str,
    organization_name: &str,
    total_shares: &str,
    total_cost: &str,
    average_cost: &str,
    current_price: &str,
    market_value: &str,
    realized_profit: &str,
    unrealized_profit: &str,
    total_profit: &str,
    total_profit_rate: &str,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE investments
         SET name = ?, organization_name = ?, total_shares = ?, total_cost = ?, average_cost = ?,
             current_price = ?, market_value = ?, realized_profit = ?, unrealized_profit = ?,
             total_profit = ?, total_profit_rate = ?, status = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(name)
    .bind(organization_name)
    .bind(total_shares)
    .bind(total_cost)
    .bind(average_cost)
    .bind(current_price)
    .bind(market_value)
    .bind(realized_profit)
    .bind(unrealized_profit)
    .bind(total_profit)
    .bind(total_profit_rate)
    .bind(status)
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_manual_metrics(
    pool: &sqlx::MySqlPool,
    id: u64,
    total_shares: &str,
    total_cost: &str,
    average_cost: &str,
    current_price: &str,
    market_value: &str,
    unrealized_profit: &str,
    total_profit: &str,
    total_profit_rate: &str,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE investments
         SET total_shares = ?, total_cost = ?, average_cost = ?, current_price = ?, market_value = ?,
             unrealized_profit = ?, total_profit = ?, total_profit_rate = ?, status = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(total_shares)
    .bind(total_cost)
    .bind(average_cost)
    .bind(current_price)
    .bind(market_value)
    .bind(unrealized_profit)
    .bind(total_profit)
    .bind(total_profit_rate)
    .bind(status)
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
