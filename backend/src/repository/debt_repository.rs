use chrono::{Datelike, NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::debt::{CreateDebtRequest, DebtListQuery, UpdateDebtRequest},
    error::app_error::AppError,
    model::debt::Debt,
};

fn map_debt(row: sqlx::mysql::MySqlRow) -> Result<Debt, sqlx::Error> {
    let start_date = row.try_get::<NaiveDate, _>("start_date")?;
    let end_date = row.try_get("end_date")?;
    let repay_deadline = row.try_get("repay_deadline")?;
    let payment_method: String = row.try_get("payment_method")?;
    let period_count: u32 = row.try_get("period_count")?;
    let period_unit: String = row.try_get("period_unit")?;
    let period_value: u32 = row.try_get("period_value")?;
    let paid_period_count = row.try_get::<i64, _>("paid_period_count")?.max(0) as u32;
    let effective_period_count = if payment_method == "credit_card" {
        period_count.max(1)
    } else {
        compute_non_credit_period_count(start_date, end_date, repay_deadline, period_count)
    };

    Ok(Debt {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        source_bill_id: row.try_get("source_bill_id")?,
        start_date,
        end_date,
        repay_deadline,
        category_id: row.try_get("category_id")?,
        category_name: row.try_get("category_name")?,
        amount: row.try_get("amount")?,
        period_count: effective_period_count,
        paid_period_count: paid_period_count.min(effective_period_count),
        period_unit,
        period_value,
        payment_method,
        status: row.try_get("status")?,
        remark: row.try_get("remark")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

fn compute_non_credit_period_count(
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    repay_deadline: Option<NaiveDate>,
    fallback_period_count: u32,
) -> u32 {
    let upper = match (end_date, repay_deadline) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    let Some(upper_bound) = upper else {
        return fallback_period_count.max(1);
    };
    if upper_bound < start_date {
        return 0;
    }

    let month_span = (upper_bound.year() - start_date.year()) * 12
        + (upper_bound.month() as i32 - start_date.month() as i32)
        + 1;
    month_span.max(1) as u32
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &DebtListQuery,
) -> Result<(Vec<Debt>, u64), AppError> {
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM debts
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR status = ?)
           AND (
                ? IS NULL
                OR (? = 'non_credit_card' AND payment_method <> 'credit_card')
                OR payment_method = ?
               )
           AND (? IS NULL OR category_name LIKE ? OR remark LIKE ? OR CAST(id AS CHAR) LIKE ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&query.payment_method)
    .bind(&query.payment_method)
    .bind(&query.payment_method)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
                "SELECT d.id, d.user_id, d.source_bill_id, d.start_date, d.end_date, d.repay_deadline, d.category_id,
                                d.category_name, CAST(d.amount AS CHAR) AS amount, d.period_count, d.period_unit, d.period_value, d.payment_method,
                                d.status, d.remark, d.created_at, d.updated_at,
                                COALESCE(b.paid_period_count, 0) AS paid_period_count
                 FROM debts d
                 LEFT JOIN (
                        SELECT related_debt_id AS debt_id, COUNT(*) AS paid_period_count
                        FROM bills
                        WHERE related_debt_id IS NOT NULL
                            AND bill_type = 'expense'
                            AND deleted_at IS NULL
                        GROUP BY related_debt_id
                 ) b ON b.debt_id = d.id
                 WHERE d.deleted_at IS NULL
                     AND (? IS NULL OR d.user_id = ?)
                     AND (? IS NULL OR d.status = ?)
                     AND (
                          ? IS NULL
                          OR (? = 'non_credit_card' AND d.payment_method <> 'credit_card')
                          OR d.payment_method = ?
                         )
                     AND (? IS NULL OR d.category_name LIKE ? OR d.remark LIKE ? OR CAST(d.id AS CHAR) LIKE ?)
                 ORDER BY d.start_date DESC, d.id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(&query.status)
    .bind(&query.status)
    .bind(&query.payment_method)
    .bind(&query.payment_method)
    .bind(&query.payment_method)
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
            .map(map_debt)
            .collect::<Result<Vec<_>, _>>()?,
        total,
    ))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Debt, AppError> {
    let row = sqlx::query(
        "SELECT d.id, d.user_id, d.source_bill_id, d.start_date, d.end_date, d.repay_deadline, d.category_id,
            d.category_name, CAST(d.amount AS CHAR) AS amount, d.period_count, d.period_unit, d.period_value, d.payment_method,
            d.status, d.remark, d.created_at, d.updated_at,
            COALESCE(b.paid_period_count, 0) AS paid_period_count
         FROM debts d
         LEFT JOIN (
            SELECT related_debt_id AS debt_id, COUNT(*) AS paid_period_count
            FROM bills
            WHERE related_debt_id IS NOT NULL
              AND bill_type = 'expense'
              AND deleted_at IS NULL
            GROUP BY related_debt_id
         ) b ON b.debt_id = d.id
         WHERE d.id = ? AND d.deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .fetch_optional(pool)
    .await?;
    row.map(map_debt).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateDebtRequest,
    category_name: &str,
    source_bill_id: Option<u64>,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "INSERT INTO debts (
            user_id, source_bill_id, start_date, end_date, repay_deadline, category_id, category_name,
            amount, period_count, period_unit, period_value, payment_method, remark
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(source_bill_id.map(parse_u64_id).transpose()?)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.repay_deadline)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(payload.period_count)
    .bind(&payload.period_unit)
    .bind(payload.period_value)
    .bind(&payload.payment_method)
    .bind(&payload.remark)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateDebtRequest,
    category_name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE debts
         SET start_date = ?, end_date = ?, repay_deadline = ?, category_id = ?, category_name = ?,
             amount = ?, period_count = ?, period_unit = ?, period_value = ?, payment_method = ?,
             status = ?, remark = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.repay_deadline)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.amount)
    .bind(payload.period_count)
    .bind(&payload.period_unit)
    .bind(payload.period_value)
    .bind(&payload.payment_method)
    .bind(&payload.status)
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
        "UPDATE debts SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(parse_u64_id(id)?)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compute_non_credit_period_count;
    use chrono::NaiveDate;

    #[test]
    fn compute_non_credit_period_count_uses_month_span_between_start_and_end() {
        let count = compute_non_credit_period_count(
            NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
            Some(NaiveDate::from_ymd_opt(2026, 4, 10).expect("valid")),
            Some(NaiveDate::from_ymd_opt(2026, 6, 10).expect("valid")),
            99,
        );
        assert_eq!(count, 4);
    }

    #[test]
    fn compute_non_credit_period_count_cross_year_month_span() {
        let count = compute_non_credit_period_count(
            NaiveDate::from_ymd_opt(2025, 11, 10).expect("valid"),
            Some(NaiveDate::from_ymd_opt(2026, 2, 9).expect("valid")),
            None,
            9,
        );
        assert_eq!(count, 4);
    }

    #[test]
    fn compute_non_credit_period_count_falls_back_when_no_boundaries() {
        let count = compute_non_credit_period_count(
            NaiveDate::from_ymd_opt(2026, 1, 10).expect("valid"),
            None,
            None,
            12,
        );
        assert_eq!(count, 12);
    }
}
