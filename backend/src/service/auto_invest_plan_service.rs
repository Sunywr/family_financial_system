use std::str::FromStr;

use chrono::Months;
use rust_decimal::Decimal;

use crate::{
    common::state::AppState,
    dto::{
        auto_invest_plan::{
            AutoInvestPlanListQuery, CreateAutoInvestPlanRequest, UpdateAutoInvestPlanRequest,
        },
        bill::CreateBillRequest,
    },
    error::app_error::AppError,
    model::auto_invest_plan::AutoInvestPlan,
    repository::{
        auto_invest_plan_repository, bill_repository, config_item_repository,
        investment_repository, user_repository,
    },
};

pub async fn list(
    state: &AppState,
    query: &AutoInvestPlanListQuery,
    auth_user_id: u64,
) -> Result<(Vec<AutoInvestPlan>, u64), AppError> {
    let pool = state.db()?;
    let auth_user = user_repository::find_by_id(pool, auth_user_id).await?;
    let effective = AutoInvestPlanListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        status: query.status.clone(),
    };
    auto_invest_plan_repository::list(pool, &effective).await
}

pub async fn detail(
    state: &AppState,
    id: u64,
    auth_user_id: u64,
) -> Result<AutoInvestPlan, AppError> {
    let plan = auto_invest_plan_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, plan.user_id).await?;
    Ok(plan)
}

pub async fn create(
    state: &AppState,
    payload: &CreateAutoInvestPlanRequest,
    auth_user_id: u64,
) -> Result<AutoInvestPlan, AppError> {
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    validate_amount(&payload.amount)?;
    if payload.cycle_months < 1 {
        return Err(AppError::BadRequest(
            "cycle_months must be at least 1".to_string(),
        ));
    }
    let pool = state.db()?;
    let investment = investment_repository::find_by_id(pool, payload.investment_id).await?;
    if investment.user_id != payload.user_id {
        return Err(AppError::BadRequest(
            "investment does not belong to this user".to_string(),
        ));
    }
    let category = config_item_repository::find_by_id(pool, payload.category_id).await?;
    let id =
        auto_invest_plan_repository::create(pool, payload, &investment.name, &category.display_name)
            .await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateAutoInvestPlanRequest,
    auth_user_id: u64,
) -> Result<AutoInvestPlan, AppError> {
    let existing = auto_invest_plan_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    validate_amount(&payload.amount)?;
    let pool = state.db()?;
    let category = config_item_repository::find_by_id(pool, payload.category_id).await?;
    auto_invest_plan_repository::update(pool, id, payload, &category.display_name).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = auto_invest_plan_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    auto_invest_plan_repository::soft_delete(state.db()?, id).await
}

/// Called by the scheduler job. Returns (plans_visited, bills_created).
pub async fn generate_for_scheduler(
    pool: &sqlx::MySqlPool,
    today: chrono::NaiveDate,
) -> Result<(usize, usize), AppError> {
    let plans = auto_invest_plan_repository::list_active_for_scheduler(pool, today).await?;
    let mut created = 0usize;
    let visited = plans.len();

    for plan in plans {
        let cycle = plan.cycle_months.max(1);
        let mut cursor = match plan.last_generated_date {
            Some(last) => last
                .checked_add_months(Months::new(cycle))
                .ok_or(AppError::Internal)?,
            None => plan.start_date,
        };

        let mut last_generated = plan.last_generated_date;

        while cursor <= today {
            if let Some(end_date) = plan.end_date {
                if cursor > end_date {
                    break;
                }
            }

            let marker = format!(
                "auto_sip_plan:{}:{}",
                plan.id,
                cursor.format("%Y%m%d")
            );
            let exists =
                bill_repository::exists_by_remark_marker(pool, plan.user_id, &marker).await?;

            if !exists {
                let remark_text = match &plan.remark {
                    Some(r) if !r.trim().is_empty() => {
                        format!("{}\n{}", r.trim(), marker)
                    }
                    _ => marker.clone(),
                };

                let req = CreateBillRequest {
                    user_id: plan.user_id,
                    account_date: cursor,
                    category_id: plan.category_id,
                    bill_type: "expense".to_string(),
                    payment_method: "cash".to_string(),
                    is_fixed_asset: false,
                    amount: plan.amount.clone(),
                    tags: None,
                    remark: Some(remark_text),
                    transfer_target_type: None,
                    transfer_target_user_id: None,
                    credit_card_id: None,
                    is_installment: None,
                    installment_months: None,
                    investment_action: Some("add_position".to_string()),
                    related_investment_id: Some(plan.investment_id),
                    product_code: None,
                    product_name: None,
                    organization_name: None,
                    share_amount: None,
                    related_asset_id: None,
                    related_debt_id: None,
                };
                bill_repository::create(pool, &req, &plan.category_name, None, "normal").await?;
                created += 1;
            }

            last_generated = Some(cursor);
            cursor = cursor
                .checked_add_months(Months::new(cycle))
                .ok_or(AppError::Internal)?;
        }

        if let Some(last) = last_generated {
            if plan.last_generated_date.map_or(true, |prev| last > prev) {
                auto_invest_plan_repository::update_last_generated_date(pool, plan.id, last)
                    .await?;
            }
        }
    }

    Ok((visited, created))
}

fn validate_amount(amount: &str) -> Result<(), AppError> {
    let d = Decimal::from_str(amount)
        .map_err(|_| AppError::BadRequest("amount must be a valid decimal".to_string()))?;
    if d <= Decimal::ZERO {
        return Err(AppError::BadRequest(
            "amount must be greater than zero".to_string(),
        ));
    }
    Ok(())
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
