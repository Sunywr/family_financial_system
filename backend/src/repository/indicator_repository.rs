use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::investment::InvestmentTopQuery,
    error::app_error::AppError,
    model::{
        investment_recommendation::InvestmentRecommendation, stock_indicator::StockIndicator,
        wealth_indicator::WealthIndicator,
    },
};

fn map_stock_indicator(row: sqlx::mysql::MySqlRow) -> Result<StockIndicator, sqlx::Error> {
    Ok(StockIndicator {
        id: row.try_get("id")?,
        investment_id: row.try_get("investment_id")?,
        user_id: row.try_get("user_id")?,
        code: row.try_get("code")?,
        indicator_date: row.try_get("indicator_date")?,
        current_price: row.try_get("current_price")?,
        price_change_rate: row.try_get("price_change_rate")?,
        profit_rate: row.try_get("profit_rate")?,
        ma_bias: row.try_get("ma_bias")?,
        volume_ratio: row.try_get("volume_ratio")?,
        score: row.try_get("score")?,
        suggestion: row.try_get("suggestion")?,
        reason: row.try_get("reason")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

fn map_wealth_indicator(row: sqlx::mysql::MySqlRow) -> Result<WealthIndicator, sqlx::Error> {
    Ok(WealthIndicator {
        id: row.try_get("id")?,
        investment_id: row.try_get("investment_id")?,
        user_id: row.try_get("user_id")?,
        code: row.try_get("code")?,
        indicator_date: row.try_get("indicator_date")?,
        current_nav: row.try_get("current_nav")?,
        annualized_return_1d: row.try_get("annualized_return_1d")?,
        annualized_return_7d: row.try_get("annualized_return_7d")?,
        annualized_return_30d: row.try_get("annualized_return_30d")?,
        drawdown_proxy: row.try_get("drawdown_proxy")?,
        profit_rate: row.try_get("profit_rate")?,
        score: row.try_get("score")?,
        suggestion: row.try_get("suggestion")?,
        reason: row.try_get("reason")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

fn map_recommendation(row: sqlx::mysql::MySqlRow) -> Result<InvestmentRecommendation, sqlx::Error> {
    Ok(InvestmentRecommendation {
        investment_id: row.try_get("investment_id")?,
        user_id: row.try_get("user_id")?,
        investment_type: row.try_get("investment_type")?,
        name: row.try_get("name")?,
        code: row.try_get("code")?,
        organization_name: row.try_get("organization_name")?,
        current_price: row.try_get("current_price")?,
        market_value: row.try_get("market_value")?,
        total_profit_rate: row.try_get("total_profit_rate")?,
        score: row.try_get("score")?,
        suggestion: row.try_get("suggestion")?,
        reason: row.try_get("reason")?,
        indicator_date: row.try_get("indicator_date")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_stock_indicator(
    pool: &sqlx::MySqlPool,
    investment_id: u64,
    user_id: u64,
    code: &str,
    indicator_date: NaiveDate,
    current_price: &str,
    price_change_rate: &str,
    profit_rate: &str,
    ma_bias: &str,
    volume_ratio: &str,
    score: i32,
    suggestion: &str,
    reason: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO stock_indicators (
            investment_id, user_id, code, indicator_date, current_price, price_change_rate,
            profit_rate, ma_bias, volume_ratio, score, suggestion, reason
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
            current_price = VALUES(current_price),
            price_change_rate = VALUES(price_change_rate),
            profit_rate = VALUES(profit_rate),
            ma_bias = VALUES(ma_bias),
            volume_ratio = VALUES(volume_ratio),
            score = VALUES(score),
            suggestion = VALUES(suggestion),
            reason = VALUES(reason),
            deleted_at = NULL",
    )
    .bind(parse_u64_id(investment_id)?)
    .bind(parse_u64_id(user_id)?)
    .bind(code)
    .bind(indicator_date)
    .bind(current_price)
    .bind(price_change_rate)
    .bind(profit_rate)
    .bind(ma_bias)
    .bind(volume_ratio)
    .bind(score)
    .bind(suggestion)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_wealth_indicator(
    pool: &sqlx::MySqlPool,
    investment_id: u64,
    user_id: u64,
    code: &str,
    indicator_date: NaiveDate,
    current_nav: &str,
    annualized_return_1d: &str,
    annualized_return_7d: &str,
    annualized_return_30d: &str,
    drawdown_proxy: &str,
    profit_rate: &str,
    score: i32,
    suggestion: &str,
    reason: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO wealth_indicators (
            investment_id, user_id, code, indicator_date, current_nav, annualized_return_1d,
            annualized_return_7d, annualized_return_30d, drawdown_proxy, profit_rate, score, suggestion, reason
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
            current_nav = VALUES(current_nav),
            annualized_return_1d = VALUES(annualized_return_1d),
            annualized_return_7d = VALUES(annualized_return_7d),
            annualized_return_30d = VALUES(annualized_return_30d),
            drawdown_proxy = VALUES(drawdown_proxy),
            profit_rate = VALUES(profit_rate),
            score = VALUES(score),
            suggestion = VALUES(suggestion),
            reason = VALUES(reason),
            deleted_at = NULL",
    )
    .bind(parse_u64_id(investment_id)?)
    .bind(parse_u64_id(user_id)?)
    .bind(code)
    .bind(indicator_date)
    .bind(current_nav)
    .bind(annualized_return_1d)
    .bind(annualized_return_7d)
    .bind(annualized_return_30d)
    .bind(drawdown_proxy)
    .bind(profit_rate)
    .bind(score)
    .bind(suggestion)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn latest_stock_indicator(
    pool: &sqlx::MySqlPool,
    investment_id: u64,
) -> Result<Option<StockIndicator>, AppError> {
    let row = sqlx::query(
        "SELECT id, investment_id, user_id, code, indicator_date,
                CAST(current_price AS CHAR) AS current_price,
                CAST(price_change_rate AS CHAR) AS price_change_rate,
                CAST(profit_rate AS CHAR) AS profit_rate,
                CAST(ma_bias AS CHAR) AS ma_bias,
                CAST(volume_ratio AS CHAR) AS volume_ratio,
                score, suggestion, reason, created_at, updated_at
         FROM stock_indicators
         WHERE investment_id = ? AND deleted_at IS NULL
         ORDER BY indicator_date DESC, id DESC
         LIMIT 1",
    )
    .bind(parse_u64_id(investment_id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_stock_indicator)
        .transpose()
        .map_err(AppError::from)
}

pub async fn latest_wealth_indicator(
    pool: &sqlx::MySqlPool,
    investment_id: u64,
) -> Result<Option<WealthIndicator>, AppError> {
    let row = sqlx::query(
        "SELECT id, investment_id, user_id, code, indicator_date,
                CAST(current_nav AS CHAR) AS current_nav,
                CAST(annualized_return_1d AS CHAR) AS annualized_return_1d,
                CAST(annualized_return_7d AS CHAR) AS annualized_return_7d,
                CAST(annualized_return_30d AS CHAR) AS annualized_return_30d,
                CAST(drawdown_proxy AS CHAR) AS drawdown_proxy,
                CAST(profit_rate AS CHAR) AS profit_rate,
                score, suggestion, reason, created_at, updated_at
         FROM wealth_indicators
         WHERE investment_id = ? AND deleted_at IS NULL
         ORDER BY indicator_date DESC, id DESC
         LIMIT 1",
    )
    .bind(parse_u64_id(investment_id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_wealth_indicator)
        .transpose()
        .map_err(AppError::from)
}

pub async fn list_top_recommendations(
    pool: &sqlx::MySqlPool,
    query: &InvestmentTopQuery,
) -> Result<Vec<InvestmentRecommendation>, AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let investment_type = query.investment_type.as_deref();
    let limit = query.limit.unwrap_or(20).min(20) as u64;

    let rows = sqlx::query(
        "SELECT *
         FROM (
            SELECT i.id AS investment_id, i.user_id, i.investment_type, i.name, i.code, i.organization_name,
                   CAST(i.current_price AS CHAR) AS current_price,
                   CAST(i.market_value AS CHAR) AS market_value,
                   CAST(i.total_profit_rate AS CHAR) AS total_profit_rate,
                   si.score, si.suggestion, si.reason, si.indicator_date
            FROM investments i
            INNER JOIN stock_indicators si
                ON si.investment_id = i.id AND si.deleted_at IS NULL
            INNER JOIN (
                SELECT investment_id, MAX(indicator_date) AS indicator_date
                FROM stock_indicators
                WHERE deleted_at IS NULL
                GROUP BY investment_id
            ) latest
                ON latest.investment_id = si.investment_id AND latest.indicator_date = si.indicator_date
            WHERE i.deleted_at IS NULL
              AND (? IS NULL OR i.user_id = ?)
              AND (? IS NULL OR i.investment_type = ?)
              AND i.investment_type = 'stock'

            UNION ALL

            SELECT i.id AS investment_id, i.user_id, i.investment_type, i.name, i.code, i.organization_name,
                   CAST(i.current_price AS CHAR) AS current_price,
                   CAST(i.market_value AS CHAR) AS market_value,
                   CAST(i.total_profit_rate AS CHAR) AS total_profit_rate,
                   wi.score, wi.suggestion, wi.reason, wi.indicator_date
            FROM investments i
            INNER JOIN wealth_indicators wi
                ON wi.investment_id = i.id AND wi.deleted_at IS NULL
            INNER JOIN (
                SELECT investment_id, MAX(indicator_date) AS indicator_date
                FROM wealth_indicators
                WHERE deleted_at IS NULL
                GROUP BY investment_id
            ) latest
                ON latest.investment_id = wi.investment_id AND latest.indicator_date = wi.indicator_date
            WHERE i.deleted_at IS NULL
              AND (? IS NULL OR i.user_id = ?)
              AND (? IS NULL OR i.investment_type = ?)
              AND i.investment_type = 'wealth'
         ) ranked
         ORDER BY score DESC, indicator_date DESC, investment_id DESC
         LIMIT ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(investment_type)
    .bind(investment_type)
    .bind(user_id)
    .bind(user_id)
    .bind(investment_type)
    .bind(investment_type)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_recommendation)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}
