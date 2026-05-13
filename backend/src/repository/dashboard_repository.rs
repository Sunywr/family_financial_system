use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::Row;

use crate::{common::id::parse_u64_id, error::app_error::AppError};

#[derive(Debug)]
struct ScheduledDebtRow {
    start_date: NaiveDate,
    repay_deadline: Option<NaiveDate>,
    amount: Decimal,
    period_unit: String,
    period_value: u32,
    payment_method: String,
}

fn parse_decimal(raw: Option<String>) -> Decimal {
    raw.and_then(|value| value.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO)
}

#[derive(Debug, Clone)]
pub struct LegacyHomepageSummary {
    pub liquid_asset: Decimal,
    pub family_total_asset: Decimal,
    pub financing_market_value: Decimal,
    pub stock_total_asset: Decimal,
    pub summary_payload: Value,
}

fn month_last_day(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("valid month");
    (next_month - Duration::days(1)).day()
}

fn shift_date_back(date: NaiveDate, unit: &str, value: u32) -> Option<NaiveDate> {
    let step = value.max(1);
    match unit {
        "day" => date.checked_sub_signed(Duration::days(i64::from(step))),
        "month" => {
            let total_months = date.year() * 12 + date.month0() as i32 - step as i32;
            let year = total_months.div_euclid(12);
            let month0 = total_months.rem_euclid(12) as u32;
            let month = month0 + 1;
            let day = date.day().min(month_last_day(year, month));
            NaiveDate::from_ymd_opt(year, month, day)
        }
        "year" => {
            let year = date.year() - step as i32;
            let day = date.day().min(month_last_day(year, date.month()));
            NaiveDate::from_ymd_opt(year, date.month(), day)
        }
        _ => None,
    }
}

fn scheduled_occurrences(
    start_date: NaiveDate,
    repay_deadline: NaiveDate,
    period_unit: &str,
    period_value: u32,
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> (u32, u32) {
    let mut total = 0_u32;
    let mut matched = 0_u32;
    let mut cursor = repay_deadline;

    while cursor >= start_date {
        total += 1;
        if cursor >= range_start && cursor <= range_end {
            matched += 1;
        }

        let Some(next_cursor) = shift_date_back(cursor, period_unit, period_value) else {
            break;
        };
        if next_cursor >= cursor {
            break;
        }
        cursor = next_cursor;
    }

    (total.max(1), matched)
}

fn debt_due_amount_in_range(
    row: &ScheduledDebtRow,
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> Decimal {
    let Some(repay_deadline) = row.repay_deadline else {
        return Decimal::ZERO;
    };
    if repay_deadline < range_start || row.start_date > range_end {
        return Decimal::ZERO;
    }

    if row.payment_method == "credit_card" {
        return if repay_deadline >= range_start && repay_deadline <= range_end {
            row.amount
        } else {
            Decimal::ZERO
        };
    }

    let (total_occurrences, matched_occurrences) = scheduled_occurrences(
        row.start_date,
        repay_deadline,
        &row.period_unit,
        row.period_value,
        range_start,
        range_end,
    );
    if matched_occurrences == 0 {
        return Decimal::ZERO;
    }

    (row.amount / Decimal::from(total_occurrences)) * Decimal::from(matched_occurrences)
}

pub async fn sum_cash_bill_amounts(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
    bill_types: &[&str],
) -> Result<Decimal, AppError> {
    let placeholders = vec!["?"; bill_types.len()].join(",");
    let sql = format!(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS CHAR) AS total
         FROM bills
         WHERE user_id = ? AND account_date >= ? AND account_date <= ? AND deleted_at IS NULL
           AND category_id <> 1
           AND payment_method NOT IN ('credit_card', 'installment')
           AND bill_type IN ({placeholders})"
    );
    let mut query = sqlx::query(&sql)
        .bind(parse_u64_id(user_id)?)
        .bind(start_date)
        .bind(end_date);
    for item in bill_types {
        query = query.bind(item);
    }
    let row = query.fetch_one(pool).await?;
    Ok(parse_decimal(row.try_get("total")?))
}

pub async fn sum_credit_card_repaid_amount(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Decimal, AppError> {
    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS CHAR) AS total
         FROM debts
         WHERE user_id = ?
           AND payment_method = 'credit_card'
           AND status = 'settled'
           AND repay_deadline >= ?
           AND repay_deadline <= ?
           AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?;
    Ok(parse_decimal(row.try_get("total")?))
}

pub async fn sum_cash_since(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Decimal, AppError> {
    let inflow = sum_cash_bill_amounts(
        pool,
        user_id,
        start_date,
        end_date,
        &["income", "refund", "reduce_position", "dividend"],
    )
    .await?;
    let outflow = sum_cash_bill_amounts(
        pool,
        user_id,
        start_date,
        end_date,
        &["expense", "open_position", "add_position"],
    )
    .await?;
    Ok(inflow - outflow)
}

pub async fn sum_index_cash_delta_since(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Decimal, AppError> {
    if start_date > end_date {
        return Ok(Decimal::ZERO);
    }

    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(
            CASE
              WHEN bill_type IN ('income', 'refund', 'reduce_position', 'dividend') THEN amount
              WHEN bill_type IN ('expense', 'open_position', 'add_position') THEN -amount
              ELSE 0
            END
         ), 0) AS CHAR) AS total
         FROM bills
         WHERE user_id = ?
           AND account_date >= ?
           AND account_date <= ?
           AND payment_method = 'cash'
           AND category_id <> 585
           AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?;

    Ok(parse_decimal(row.try_get("total")?))
}

pub async fn sum_investments_by_type(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    investment_type: &str,
) -> Result<Decimal, AppError> {
    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(market_value), 0) AS CHAR) AS total
         FROM investments
         WHERE user_id = ? AND investment_type = ? AND deleted_at IS NULL AND status = 'holding'",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(investment_type)
    .fetch_one(pool)
    .await?;
    Ok(parse_decimal(row.try_get("total")?))
}

pub async fn stock_idle_cash(pool: &sqlx::MySqlPool, user_id: u64) -> Result<Decimal, AppError> {
    let row = sqlx::query(
        "SELECT content
         FROM intel_items
         WHERE user_id = ?
           AND source = 'pfm_stock_account'
           AND deleted_at IS NULL
         ORDER BY item_date DESC, id DESC
         LIMIT 1",
    )
    .bind(parse_u64_id(user_id)?)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(Decimal::ZERO);
    };
    let content: Option<String> = row.try_get("content")?;
    let Some(content) = content else {
        return Ok(Decimal::ZERO);
    };
    let parsed: Value = serde_json::from_str(&content).unwrap_or(Value::Null);
    let amount = parsed
        .get("available_cash")
        .or_else(|| parsed.get("total_cash"))
        .and_then(Value::as_f64)
        .map(|value| Decimal::try_from(value).unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);
    Ok(amount.round_dp(2))
}

pub async fn latest_legacy_homepage_summary(
    pool: &sqlx::MySqlPool,
    user_id: u64,
) -> Result<Option<LegacyHomepageSummary>, AppError> {
    let row = sqlx::query(
        "SELECT content
         FROM intel_items
         WHERE user_id = ?
           AND source = 'pfm_homepage_summary'
           AND deleted_at IS NULL
         ORDER BY item_date DESC, id DESC
         LIMIT 1",
    )
    .bind(parse_u64_id(user_id)?)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let content: Option<String> = row.try_get("content")?;
    let Some(content) = content else {
        return Ok(None);
    };
    let parsed: Value = serde_json::from_str(&content).unwrap_or(Value::Null);
    if parsed.is_null() {
        return Ok(None);
    }

    let to_decimal = |key: &str| -> Decimal {
        match parsed.get(key) {
            Some(Value::Number(num)) => num
                .as_f64()
                .and_then(|value| Decimal::try_from(value).ok())
                .unwrap_or(Decimal::ZERO),
            Some(Value::String(text)) => text.parse::<Decimal>().unwrap_or(Decimal::ZERO),
            _ => Decimal::ZERO,
        }
    };

    Ok(Some(LegacyHomepageSummary {
        liquid_asset: to_decimal("liquid_asset").round_dp(2),
        family_total_asset: to_decimal("family_total_asset").round_dp(2),
        financing_market_value: to_decimal("financing_market_value").round_dp(2),
        stock_total_asset: to_decimal("stock_total_asset").round_dp(2),
        summary_payload: parsed,
    }))
}

async fn list_pending_debts(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    payment_method: Option<&str>,
) -> Result<Vec<ScheduledDebtRow>, AppError> {
    let rows = sqlx::query(
        "SELECT start_date, repay_deadline, CAST(amount AS CHAR) AS amount, period_unit, period_value, payment_method
         FROM debts
         WHERE user_id = ?
           AND status = 'pending'
           AND deleted_at IS NULL
           AND (? IS NULL OR payment_method = ?)",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(payment_method)
    .bind(payment_method)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let raw_amount: String = row.try_get("amount")?;
            Ok(ScheduledDebtRow {
                start_date: row.try_get("start_date")?,
                repay_deadline: row.try_get("repay_deadline")?,
                amount: raw_amount.parse::<Decimal>().unwrap_or(Decimal::ZERO),
                period_unit: row.try_get("period_unit")?,
                period_value: row.try_get::<u32, _>("period_value")?.max(1),
                payment_method: row.try_get("payment_method")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::from)
}

pub async fn sum_credit_card_outstanding(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Decimal, AppError> {
    let rows = list_pending_debts(pool, user_id, Some("credit_card")).await?;
    Ok(rows
        .iter()
        .map(|row| debt_due_amount_in_range(row, start_date, end_date))
        .sum())
}

pub async fn sum_scheduled_debt_outstanding(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Decimal, AppError> {
    let rows = list_pending_debts(pool, user_id, None).await?;
    Ok(rows
        .iter()
        .filter(|row| row.payment_method != "credit_card")
        .map(|row| debt_due_amount_in_range(row, start_date, end_date))
        .sum())
}

pub async fn daily_cash_deltas(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(NaiveDate, Decimal)>, AppError> {
    let rows = sqlx::query(
        "SELECT trend_date,
                CAST(COALESCE(SUM(net_amount), 0) AS CHAR) AS net_amount
         FROM (
            SELECT account_date AS trend_date,
                   CASE
                     WHEN bill_type IN ('income','refund','reduce_position','dividend') THEN amount
                     WHEN bill_type IN ('expense','open_position','add_position') THEN -amount
                     ELSE 0
                   END AS net_amount
            FROM bills
            WHERE user_id = ?
              AND account_date >= ?
              AND account_date <= ?
              AND category_id <> 1
              AND payment_method NOT IN ('credit_card', 'installment')
              AND deleted_at IS NULL
         ) merged
         GROUP BY trend_date
         ORDER BY trend_date ASC",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let raw_amount: String = row.try_get("net_amount")?;
            Ok((
                row.try_get("trend_date")?,
                raw_amount.parse::<Decimal>().unwrap_or(Decimal::ZERO),
            ))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::from)
}

pub async fn daily_index_cash_deltas(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(NaiveDate, Decimal)>, AppError> {
    if start_date > end_date {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        "SELECT trend_date,
                CAST(COALESCE(SUM(net_amount), 0) AS CHAR) AS net_amount
         FROM (
            SELECT account_date AS trend_date,
                   CASE
                     WHEN bill_type IN ('income','refund','reduce_position','dividend') THEN amount
                     WHEN bill_type IN ('expense','open_position','add_position') THEN -amount
                     ELSE 0
                   END AS net_amount
            FROM bills
            WHERE user_id = ?
              AND account_date >= ?
              AND account_date <= ?
              AND payment_method = 'cash'
              AND category_id <> 585
              AND deleted_at IS NULL
         ) merged
         GROUP BY trend_date
         ORDER BY trend_date ASC",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let raw_amount: String = row.try_get("net_amount")?;
            Ok((
                row.try_get("trend_date")?,
                raw_amount.parse::<Decimal>().unwrap_or(Decimal::ZERO),
            ))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::from)
}

pub async fn daily_debt_repayment_deltas(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(NaiveDate, Decimal)>, AppError> {
    if start_date > end_date {
        return Ok(Vec::new());
    }

    let mut deltas: Vec<(NaiveDate, Decimal)> = Vec::new();

    let credit_rows = sqlx::query(
        "SELECT repay_deadline AS trend_date,
                CAST(COALESCE(SUM(amount), 0) AS CHAR) AS amount
         FROM debts
         WHERE user_id = ?
           AND payment_method = 'credit_card'
           AND status = 'settled'
           AND repay_deadline >= ?
           AND repay_deadline <= ?
           AND deleted_at IS NULL
         GROUP BY repay_deadline
         ORDER BY repay_deadline ASC",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    for row in credit_rows {
        let amount: String = row.try_get("amount")?;
        let repay = amount.parse::<Decimal>().unwrap_or(Decimal::ZERO);
        if repay.is_zero() {
            continue;
        }
        deltas.push((row.try_get("trend_date")?, -repay));
    }

    let month_start =
        NaiveDate::from_ymd_opt(end_date.year(), end_date.month(), 1).expect("valid month start");
    let latest_cycle_repay_date = sqlx::query(
        "SELECT MAX(repay_deadline) AS latest_repay_date
         FROM debts
         WHERE user_id = ?
           AND payment_method <> 'credit_card'
           AND status = 'settled'
           AND repay_deadline >= ?
           AND repay_deadline <= ?
           AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(user_id)?)
    .bind(month_start)
    .bind(end_date)
    .fetch_one(pool)
    .await?
    .try_get::<Option<NaiveDate>, _>("latest_repay_date")?;

    if let Some(latest_date) = latest_cycle_repay_date
        && latest_date >= start_date
        && latest_date <= end_date
    {
        let row = sqlx::query(
            "SELECT CAST(COALESCE(SUM(amount), 0) AS CHAR) AS amount
             FROM debts
             WHERE user_id = ?
               AND payment_method <> 'credit_card'
               AND status = 'settled'
               AND repay_deadline = ?
               AND deleted_at IS NULL",
        )
        .bind(parse_u64_id(user_id)?)
        .bind(latest_date)
        .fetch_one(pool)
        .await?;
        let amount: String = row.try_get("amount")?;
        let repay = amount.parse::<Decimal>().unwrap_or(Decimal::ZERO);
        if !repay.is_zero() {
            deltas.push((latest_date, -repay));
        }
    }

    Ok(deltas)
}

pub async fn latest_credit_card_repay_deadline(
    pool: &sqlx::MySqlPool,
    user_id: u64,
) -> Result<Option<NaiveDate>, AppError> {
    let row = sqlx::query(
        "SELECT MAX(repay_deadline) AS latest_repay_date
         FROM debts
         WHERE user_id = ?
           AND repay_deadline IS NOT NULL
           AND payment_method = 'credit_card'
           AND status <> 'cancelled'
           AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(user_id)?)
    .fetch_one(pool)
    .await?;

    row.try_get::<Option<NaiveDate>, _>("latest_repay_date")
        .map_err(AppError::from)
}
