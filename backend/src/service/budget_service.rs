use std::str::FromStr;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sqlx::Row;

use crate::{
    common::state::AppState,
    dto::budget::{
        BudgetListQuery, CreateBudgetRequest, GenerateBudgetRequest, UpdateBudgetRequest,
    },
    error::app_error::AppError,
    model::{budget::Budget, config_item::ConfigItem},
    repository::{budget_repository, config_item_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &BudgetListQuery,
    auth_user_id: u64,
) -> Result<(Vec<Budget>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = BudgetListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        budget_month: query.budget_month,
    };
    budget_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(state: &AppState, id: u64, auth_user_id: u64) -> Result<Budget, AppError> {
    let budget = budget_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, budget.user_id).await?;
    Ok(budget)
}

pub async fn create(
    state: &AppState,
    payload: &CreateBudgetRequest,
    auth_user_id: u64,
) -> Result<Budget, AppError> {
    let category = validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = budget_repository::create(state.db()?, payload, &category.display_name, false).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBudgetRequest,
    auth_user_id: u64,
) -> Result<Budget, AppError> {
    let existing = budget_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    let category = validate_update(state, payload).await?;
    budget_repository::update(state.db()?, id, payload, &category.display_name).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = budget_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    budget_repository::soft_delete(state.db()?, id).await
}

pub async fn generate(
    state: &AppState,
    payload: &GenerateBudgetRequest,
    auth_user_id: u64,
) -> Result<usize, AppError> {
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    let month_start = normalize_month(payload.budget_month)?;
    let categories = config_item_repository::list(
        state.db()?,
        &crate::dto::config_item::ConfigItemListQuery {
            pagination: crate::common::pagination::PaginationQuery {
                page: 1,
                page_size: 200,
            },
            config_type: Some("account_category".to_string()),
            keyword: None,
            enabled: Some(true),
        },
    )
    .await?
    .0;

    let mut affected = 0usize;
    for category in categories {
        if ["stock", "wealth", "transfer"].contains(&category.name.as_str()) {
            continue;
        }
        let planned =
            compute_budget_amount(state.db()?, payload.user_id, month_start, &category).await?;
        budget_repository::upsert_generated(
            state.db()?,
            payload.user_id,
            month_start,
            category.id,
            &category.display_name,
            &format_decimal2(planned),
        )
        .await?;
        affected += 1;
    }

    Ok(affected)
}

pub async fn generate_for_scheduler(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    month_start: NaiveDate,
) -> Result<usize, AppError> {
    let categories = config_item_repository::list(
        pool,
        &crate::dto::config_item::ConfigItemListQuery {
            pagination: crate::common::pagination::PaginationQuery {
                page: 1,
                page_size: 200,
            },
            config_type: Some("account_category".to_string()),
            keyword: None,
            enabled: Some(true),
        },
    )
    .await?
    .0;

    let mut affected = 0usize;
    for category in categories {
        if ["stock", "wealth", "transfer"].contains(&category.name.as_str()) {
            continue;
        }
        let planned = compute_budget_amount(pool, user_id, month_start, &category).await?;
        budget_repository::upsert_generated(
            pool,
            user_id,
            month_start,
            category.id,
            &category.display_name,
            &format_decimal2(planned),
        )
        .await?;
        affected += 1;
    }

    Ok(affected)
}

async fn compute_budget_amount(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    month_start: NaiveDate,
    category: &ConfigItem,
) -> Result<Decimal, AppError> {
    let prev_month = shift_month(month_start, -1)?;
    let last_year_same_month =
        NaiveDate::from_ymd_opt(month_start.year() - 1, month_start.month(), 1)
            .ok_or_else(|| AppError::BadRequest("invalid budget month".to_string()))?;

    let prev_sum = expense_sum_for_month(pool, user_id, prev_month, category.id).await?;
    let year_sum = expense_sum_for_month(pool, user_id, last_year_same_month, category.id).await?;

    let planned = match (prev_sum > Decimal::ZERO, year_sum > Decimal::ZERO) {
        (true, true) => (prev_sum + year_sum) / Decimal::from(2u32),
        (true, false) => prev_sum,
        (false, true) => year_sum,
        (false, false) => Decimal::ZERO,
    };

    Ok(planned.round_dp(2))
}

async fn expense_sum_for_month(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    month_start: NaiveDate,
    category_id: u64,
) -> Result<Decimal, AppError> {
    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS CHAR) AS total
         FROM bills
         WHERE user_id = ?
           AND category_id = ?
           AND bill_type IN ('expense')
           AND DATE_FORMAT(account_date, '%Y-%m-01') = ?
           AND deleted_at IS NULL",
    )
    .bind(crate::common::id::parse_u64_id(user_id)?)
    .bind(crate::common::id::parse_u64_id(category_id)?)
    .bind(month_start)
    .fetch_one(pool)
    .await?;
    let raw: String = row.try_get("total")?;
    Decimal::from_str(&raw).map_err(|_| AppError::internal_with_log("invalid budget sum"))
}

async fn validate_create(
    state: &AppState,
    payload: &CreateBudgetRequest,
) -> Result<ConfigItem, AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_budget_fields(&category, &payload.planned_amount, payload.budget_month)?;
    Ok(category)
}

async fn validate_update(
    state: &AppState,
    payload: &UpdateBudgetRequest,
) -> Result<ConfigItem, AppError> {
    let category = config_item_repository::find_by_id(state.db()?, payload.category_id).await?;
    validate_budget_fields(&category, &payload.planned_amount, payload.budget_month)?;
    Ok(category)
}

fn validate_budget_fields(
    category: &ConfigItem,
    planned_amount: &str,
    budget_month: NaiveDate,
) -> Result<(), AppError> {
    if category.config_type != "account_category" && category.config_type != "budget_category" {
        return Err(AppError::BadRequest(
            "budget category must be account_category or budget_category".to_string(),
        ));
    }
    let _ = normalize_month(budget_month)?;
    let amount = Decimal::from_str(planned_amount)
        .map_err(|_| AppError::BadRequest("planned_amount must be a valid decimal".to_string()))?;
    if amount < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "planned_amount must not be negative".to_string(),
        ));
    }
    Ok(())
}

fn normalize_month(value: NaiveDate) -> Result<NaiveDate, AppError> {
    NaiveDate::from_ymd_opt(value.year(), value.month(), 1)
        .ok_or_else(|| AppError::BadRequest("invalid budget_month".to_string()))
}

fn shift_month(month_start: NaiveDate, delta: i32) -> Result<NaiveDate, AppError> {
    let total_months = month_start.year() * 12 + month_start.month0() as i32 + delta;
    let year = total_months.div_euclid(12);
    let month0 = total_months.rem_euclid(12) as u32;
    NaiveDate::from_ymd_opt(year, month0 + 1, 1)
        .ok_or_else(|| AppError::BadRequest("invalid month shift".to_string()))
}

fn format_decimal2(value: Decimal) -> String {
    value.round_dp(2).to_string()
}

async fn ensure_owner_access(
    state: &AppState,
    auth_user_id: u64,
    owner_user_id: u64,
) -> Result<(), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    if auth_user.username != "admin" && owner_user_id != auth_user_id {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
