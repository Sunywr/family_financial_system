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
    let investment_id = query.investment_id.map(parse_u64_id).transpose()?;
    let source_bill_id = query.source_bill_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM investment_transactions
         WHERE deleted_at IS NULL
           AND (? IS NULL OR investment_id = ?)
           AND (? IS NULL OR source_bill_id = ?)
           AND (? IS NULL OR action LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)",
    )
    .bind(investment_id)
    .bind(investment_id)
    .bind(source_bill_id)
    .bind(source_bill_id)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, investment_id, source_bill_id, transaction_date, action,
                CAST(shares AS CHAR) AS shares,
                CAST(amount AS CHAR) AS amount,
                CAST(unit_price AS CHAR) AS unit_price,
                CAST(realized_profit AS CHAR) AS realized_profit,
                remark, created_at, updated_at
         FROM investment_transactions
         WHERE deleted_at IS NULL
           AND (? IS NULL OR investment_id = ?)
           AND (? IS NULL OR source_bill_id = ?)
           AND (? IS NULL OR action LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)
         ORDER BY transaction_date DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(investment_id)
    .bind(investment_id)
    .bind(source_bill_id)
    .bind(source_bill_id)
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
