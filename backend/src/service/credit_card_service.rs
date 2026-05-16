use crate::{
    common::state::AppState,
    dto::credit_card::{CreateCreditCardRequest, CreditCardListQuery, UpdateCreditCardRequest},
    error::app_error::AppError,
    model::credit_card::CreditCard,
    repository::{credit_card_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &CreditCardListQuery,
    auth_user_id: u64,
) -> Result<(Vec<CreditCard>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = CreditCardListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        enabled: query.enabled,
    };
    credit_card_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(
    state: &AppState,
    id: u64,
    auth_user_id: u64,
) -> Result<CreditCard, AppError> {
    let card = credit_card_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, card.user_id).await?;
    Ok(card)
}

pub async fn create(
    state: &AppState,
    payload: &CreateCreditCardRequest,
    auth_user_id: u64,
) -> Result<CreditCard, AppError> {
    validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = credit_card_repository::create(state.db()?, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateCreditCardRequest,
    auth_user_id: u64,
) -> Result<CreditCard, AppError> {
    let existing = credit_card_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    validate_update(payload)?;
    validate_days(payload.billing_day, payload.repayment_day)?;
    credit_card_repository::update(state.db()?, id, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = credit_card_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    let active_bill_count = credit_card_repository::count_active_bills(state.db()?, id).await?;
    if active_bill_count > 0 {
        return Err(AppError::BadRequest(format!(
            "cannot delete credit card with {active_bill_count} active bills"
        )));
    }
    credit_card_repository::soft_delete(state.db()?, id).await
}

async fn validate_create(
    state: &AppState,
    payload: &CreateCreditCardRequest,
) -> Result<(), AppError> {
    if payload.name.trim().is_empty() || payload.credit_limit.trim().is_empty() {
        return Err(AppError::BadRequest(
            "name and credit_limit are required".to_string(),
        ));
    }
    validate_days(payload.billing_day, payload.repayment_day)?;
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    Ok(())
}

fn validate_update(payload: &UpdateCreditCardRequest) -> Result<(), AppError> {
    if payload.name.trim().is_empty() || payload.credit_limit.trim().is_empty() {
        return Err(AppError::BadRequest(
            "name and credit_limit are required".to_string(),
        ));
    }
    Ok(())
}

fn validate_days(billing_day: u8, repayment_day: u8) -> Result<(), AppError> {
    if !(1..=31).contains(&billing_day) || !(1..=31).contains(&repayment_day) {
        return Err(AppError::BadRequest(
            "billing_day and repayment_day must be between 1 and 31".to_string(),
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
