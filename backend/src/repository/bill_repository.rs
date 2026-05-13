use chrono::{NaiveDate, NaiveDateTime};
use sqlx::Row;

use crate::{
    common::id::parse_u64_id,
    dto::bill::{BillListQuery, CreateBillRequest, UpdateBillRequest},
    error::app_error::AppError,
    model::bill::Bill,
};

fn parse_json_tags(raw: Option<String>) -> Vec<String> {
    raw.and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())
        .unwrap_or_default()
}

fn map_bill(row: sqlx::mysql::MySqlRow) -> Result<Bill, sqlx::Error> {
    Ok(Bill {
        id: row.try_get::<u64, _>("id")?,
        user_id: row.try_get::<u64, _>("user_id")?,
        account_date: row.try_get::<NaiveDate, _>("account_date")?,
        category_id: row.try_get::<u64, _>("category_id")?,
        category_name: row.try_get("category_name")?,
        bill_type: row.try_get("bill_type")?,
        payment_method: row.try_get("payment_method")?,
        is_fixed_asset: row.try_get::<bool, _>("is_fixed_asset")?,
        amount: row.try_get("amount")?,
        tags: parse_json_tags(row.try_get::<Option<String>, _>("tags")?),
        remark: row.try_get("remark")?,
        transfer_group_id: row.try_get("transfer_group_id")?,
        transfer_target_type: row.try_get("transfer_target_type")?,
        transfer_target_user_id: row.try_get("transfer_target_user_id")?,
        credit_card_id: row.try_get("credit_card_id")?,
        is_installment: row.try_get::<bool, _>("is_installment")?,
        installment_months: row.try_get("installment_months")?,
        investment_action: row.try_get("investment_action")?,
        related_investment_id: row.try_get("related_investment_id")?,
        product_code: row.try_get("product_code")?,
        product_name: row.try_get("product_name")?,
        organization_name: row.try_get("organization_name")?,
        share_amount: row.try_get("share_amount")?,
        related_asset_id: row.try_get("related_asset_id")?,
        special_status: row.try_get("special_status")?,
        created_at: row.try_get::<NaiveDateTime, _>("created_at")?,
        updated_at: row.try_get::<NaiveDateTime, _>("updated_at")?,
    })
}

fn serialize_tags(tags: Option<&[String]>) -> Result<Option<String>, AppError> {
    tags.map(serde_json::to_string)
        .transpose()
        .map_err(AppError::internal_with_log)
}

pub async fn list(
    pool: &sqlx::MySqlPool,
    query: &BillListQuery,
) -> Result<(Vec<Bill>, u64), AppError> {
    let keyword = query.keyword.as_ref().map(|value| format!("%{value}%"));
    let user_id = query.user_id.map(parse_u64_id).transpose()?;
    let category_id = query.category_id.map(parse_u64_id).transpose()?;

    let total_row = sqlx::query(
        "SELECT COUNT(*) AS total
         FROM bills
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR category_id = ?)
           AND (? IS NULL OR bill_type = ?)
           AND (? IS NULL OR payment_method = ?)
           AND (? IS NULL OR account_date >= ?)
           AND (? IS NULL OR account_date <= ?)
           AND (? IS NULL OR category_name LIKE ? OR remark LIKE ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(category_id)
    .bind(category_id)
    .bind(&query.bill_type)
    .bind(&query.bill_type)
    .bind(&query.payment_method)
    .bind(&query.payment_method)
    .bind(query.start_date)
    .bind(query.start_date)
    .bind(query.end_date)
    .bind(query.end_date)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .fetch_one(pool)
    .await?;
    let total = total_row.try_get::<i64, _>("total")?.max(0) as u64;

    let rows = sqlx::query(
        "SELECT id, user_id, account_date, category_id, category_name, bill_type, payment_method,
                is_fixed_asset, CAST(amount AS CHAR) AS amount, CAST(tags AS CHAR) AS tags, remark, transfer_group_id,
                transfer_target_type, transfer_target_user_id, credit_card_id, is_installment,
                installment_months, investment_action, related_investment_id, product_code,
                product_name, organization_name, CAST(share_amount AS CHAR) AS share_amount, related_asset_id, special_status,
                created_at, updated_at
         FROM bills
         WHERE deleted_at IS NULL
           AND (? IS NULL OR user_id = ?)
           AND (? IS NULL OR category_id = ?)
           AND (? IS NULL OR bill_type = ?)
           AND (? IS NULL OR payment_method = ?)
           AND (? IS NULL OR account_date >= ?)
           AND (? IS NULL OR account_date <= ?)
           AND (? IS NULL OR category_name LIKE ? OR remark LIKE ?)
         ORDER BY account_date DESC, id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(category_id)
    .bind(category_id)
    .bind(&query.bill_type)
    .bind(&query.bill_type)
    .bind(&query.payment_method)
    .bind(&query.payment_method)
    .bind(query.start_date)
    .bind(query.start_date)
    .bind(query.end_date)
    .bind(query.end_date)
    .bind(&keyword)
    .bind(&keyword)
    .bind(&keyword)
    .bind(query.pagination.page_size)
    .bind(query.pagination.offset())
    .fetch_all(pool)
    .await?;

    let list = rows
        .into_iter()
        .map(map_bill)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((list, total))
}

pub async fn find_by_id(pool: &sqlx::MySqlPool, id: u64) -> Result<Bill, AppError> {
    let id = parse_u64_id(id)?;
    let row = sqlx::query(
        "SELECT id, user_id, account_date, category_id, category_name, bill_type, payment_method,
                is_fixed_asset, amount, CAST(tags AS CHAR) AS tags, remark, transfer_group_id,
                transfer_target_type, transfer_target_user_id, credit_card_id, is_installment,
                installment_months, investment_action, related_investment_id, product_code,
                product_name, organization_name, share_amount, related_asset_id, special_status,
                created_at, updated_at
         FROM bills
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    row.map(map_bill).transpose()?.ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &sqlx::MySqlPool,
    payload: &CreateBillRequest,
    category_name: &str,
    transfer_group_id: Option<&str>,
    special_status: &str,
) -> Result<u64, AppError> {
    let tags_json = serialize_tags(payload.tags.as_deref())?;
    let result = sqlx::query(
        "INSERT INTO bills (
            user_id, account_date, category_id, category_name, bill_type, payment_method,
            is_fixed_asset, amount, tags, remark, transfer_group_id, transfer_target_type,
            transfer_target_user_id, credit_card_id, is_installment, installment_months,
            investment_action, related_investment_id, product_code, product_name,
            organization_name, share_amount, related_asset_id, special_status
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CAST(? AS JSON), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(parse_u64_id(payload.user_id)?)
    .bind(payload.account_date)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.bill_type)
    .bind(&payload.payment_method)
    .bind(payload.is_fixed_asset)
    .bind(&payload.amount)
    .bind(tags_json)
    .bind(&payload.remark)
    .bind(transfer_group_id)
    .bind(&payload.transfer_target_type)
    .bind(payload.transfer_target_user_id.map(parse_u64_id).transpose()?)
    .bind(payload.credit_card_id.map(parse_u64_id).transpose()?)
    .bind(payload.is_installment.unwrap_or(false))
    .bind(payload.installment_months)
    .bind(&payload.investment_action)
    .bind(payload.related_investment_id.map(parse_u64_id).transpose()?)
    .bind(&payload.product_code)
    .bind(&payload.product_name)
    .bind(&payload.organization_name)
    .bind(&payload.share_amount)
    .bind(payload.related_asset_id.map(parse_u64_id).transpose()?)
    .bind(special_status)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn create_transfer_mirror(
    pool: &sqlx::MySqlPool,
    payload: &CreateBillRequest,
    target_user_id: u64,
    category_name: &str,
    transfer_group_id: &str,
) -> Result<u64, AppError> {
    let tags_json = serialize_tags(payload.tags.as_deref())?;
    let result = sqlx::query(
        "INSERT INTO bills (
            user_id, account_date, category_id, category_name, bill_type, payment_method,
            is_fixed_asset, amount, tags, remark, transfer_group_id, transfer_target_type,
            transfer_target_user_id, is_installment, special_status
         ) VALUES (?, ?, ?, ?, ?, 'cash', 0, ?, CAST(? AS JSON), ?, ?, 'system_user', ?, 0, 'transfer_mirror')",
    )
    .bind(parse_u64_id(target_user_id)?)
    .bind(payload.account_date)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind("income")
    .bind(&payload.amount)
    .bind(tags_json)
    .bind(&payload.remark)
    .bind(transfer_group_id)
    .bind(parse_u64_id(payload.user_id)?)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn update(
    pool: &sqlx::MySqlPool,
    id: u64,
    payload: &UpdateBillRequest,
    category_name: &str,
    special_status: &str,
) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let tags_json = serialize_tags(payload.tags.as_deref())?;
    let result = sqlx::query(
        "UPDATE bills
         SET account_date = ?, category_id = ?, category_name = ?, bill_type = ?, payment_method = ?,
             is_fixed_asset = ?, amount = ?, tags = CAST(? AS JSON), remark = ?, transfer_target_type = ?,
             transfer_target_user_id = ?, credit_card_id = ?, is_installment = ?, installment_months = ?,
             investment_action = ?, related_investment_id = ?, product_code = ?, product_name = ?,
             organization_name = ?, share_amount = ?, related_asset_id = ?, special_status = ?
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(payload.account_date)
    .bind(parse_u64_id(payload.category_id)?)
    .bind(category_name)
    .bind(&payload.bill_type)
    .bind(&payload.payment_method)
    .bind(payload.is_fixed_asset)
    .bind(&payload.amount)
    .bind(tags_json)
    .bind(&payload.remark)
    .bind(&payload.transfer_target_type)
    .bind(payload.transfer_target_user_id.map(parse_u64_id).transpose()?)
    .bind(payload.credit_card_id.map(parse_u64_id).transpose()?)
    .bind(payload.is_installment.unwrap_or(false))
    .bind(payload.installment_months)
    .bind(&payload.investment_action)
    .bind(payload.related_investment_id.map(parse_u64_id).transpose()?)
    .bind(&payload.product_code)
    .bind(&payload.product_name)
    .bind(&payload.organization_name)
    .bind(&payload.share_amount)
    .bind(payload.related_asset_id.map(parse_u64_id).transpose()?)
    .bind(special_status)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn soft_delete(pool: &sqlx::MySqlPool, id: u64) -> Result<(), AppError> {
    let id = parse_u64_id(id)?;
    let result = sqlx::query(
        "UPDATE bills SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
