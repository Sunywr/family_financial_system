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
) -> Result<(Vec<CreditCard>, u64), AppError> {
    credit_card_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<CreditCard, AppError> {
    credit_card_repository::find_by_id(state.db()?, id).await
}

pub async fn create(
    state: &AppState,
    payload: &CreateCreditCardRequest,
) -> Result<CreditCard, AppError> {
    validate_create(state, payload).await?;
    let id = credit_card_repository::create(state.db()?, payload).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateCreditCardRequest,
) -> Result<CreditCard, AppError> {
    validate_days(payload.billing_day, payload.repayment_day)?;
    credit_card_repository::update(state.db()?, id, payload).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
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

fn validate_days(billing_day: u8, repayment_day: u8) -> Result<(), AppError> {
    if !(1..=31).contains(&billing_day) || !(1..=31).contains(&repayment_day) {
        return Err(AppError::BadRequest(
            "billing_day and repayment_day must be between 1 and 31".to_string(),
        ));
    }
    Ok(())
}
