use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id, dto::investment_transaction::InvestmentTransactionListQuery,
    error::app_error::AppError, model::investment_transaction::InvestmentTransaction,
};

fn map_tx(row: sqlx::mysql::MySqlRow) -> Result<InvestmentTransaction, sqlx::Error> {
    Ok(InvestmentTransaction {
        id: row.try_get("id")?,
        investment_id: row.try_get("investment_id")?,
        investment_name: row.try_get("investment_name")?,
        investment_code: row.try_get("investment_code")?,
        source_bill_id: row.try_get("source_bill_id")?,
        transaction_date: row.try_get::<NaiveDate, _>("transaction_date")?,
        action: row.try_get("action")?,
        shares: row.try_get("shares")?,
        amount: row.try_get("amount")?,
        unit_price: row.try_get("unit_price")?,
        realized_profit: row.try_get("realized_profit")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &InvestmentTransactionListQuery,
) -> Result<(Vec<InvestmentTransaction>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let investment_id = query.investment_id.map(parse_u64_id).transpose()?;
    let source_bill_id = query.source_bill_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM investment_transactions it
         LEFT JOIN investments i ON i.id = it.investment_id
         WHERE it.deleted_at IS NULL
                     AND (? IS NULL OR i.user_id = ?)
           AND (? IS NULL OR it.investment_id = ?)
           AND (? IS NULL OR it.source_bill_id = ?)
           AND (
             ? IS NULL
             OR it.action LIKE ?
             OR it.remark LIKE ?
             OR i.name LIKE ?
             OR i.code LIKE ?
             OR CAST(it.id AS CHAR) LIKE ?
             OR CAST(it.source_bill_id AS CHAR) LIKE ?
           )",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(investment_id)
    .bind(investment_id)
    .bind(source_bill_id)
    .bind(source_bill_id)
    .bind(&keyword)
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
        "SELECT it.id, it.investment_id, i.name AS investment_name, i.code AS investment_code,
                it.source_bill_id, it.transaction_date, it.action,
                CAST(it.shares AS CHAR) AS shares,
                CAST(it.amount AS CHAR) AS amount,
                CAST(it.unit_price AS CHAR) AS unit_price,
                CAST(it.realized_profit AS CHAR) AS realized_profit,
                it.remark, it.created_at, it.updated_at
         FROM investment_transactions it
         LEFT JOIN investments i ON i.id = it.investment_id
         WHERE it.deleted_at IS NULL
                     AND (? IS NULL OR i.user_id = ?)
           AND (? IS NULL OR it.investment_id = ?)
           AND (? IS NULL OR it.source_bill_id = ?)
           AND (
             ? IS NULL
             OR it.action LIKE ?
             OR it.remark LIKE ?
             OR i.name LIKE ?
             OR i.code LIKE ?
             OR CAST(it.id AS CHAR) LIKE ?
             OR CAST(it.source_bill_id AS CHAR) LIKE ?
           )
         ORDER BY it.transaction_date DESC, it.id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(investment_id)
    .bind(investment_id)
    .bind(source_bill_id)
    .bind(source_bill_id)
    .bind(&keyword)
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
            .map(map_tx)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &sqlx::MySqlPool,
    investment_id: u64,
    source_bill_id: u64,
    transaction_date: NaiveDate,
    action: &str,
    shares: &str,
    amount: &str,
    unit_price: &str,
    realized_profit: &str,
    remark: Option<&str>,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO investment_transactions (
            investment_id, source_bill_id, transaction_date, action, shares, amount, unit_price,
            realized_profit, remark
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(investment_id)?)
    .bind(parse_u64_id(source_bill_id)?)
    .bind(transaction_date)
    .bind(action)
    .bind(shares)
    .bind(amount)
    .bind(unit_price)
    .bind(realized_profit)
    .bind(remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}
