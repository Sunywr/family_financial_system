use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};

use crate::{
    common::{
        response::{ok, paged},
        state::AppState,
    },
    dto::credit_card::{CreateCreditCardRequest, CreditCardListQuery, UpdateCreditCardRequest},
    error::app_error::AppError,
    service::credit_card_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<CreditCardListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = credit_card_service::list(&state, &query).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(credit_card_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateCreditCardRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(credit_card_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateCreditCardRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(credit_card_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    credit_card_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
