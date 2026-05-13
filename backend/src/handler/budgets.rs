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
    dto::budget::{
        BudgetListQuery, CreateBudgetRequest, GenerateBudgetRequest, UpdateBudgetRequest,
    },
    error::app_error::AppError,
    service::budget_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<BudgetListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = budget_service::list(&state, &query).await?;
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
    Ok(ok(budget_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateBudgetRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(budget_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateBudgetRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(budget_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    budget_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

pub async fn generate(
    State(state): State<AppState>,
    Json(payload): Json<GenerateBudgetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let generated = budget_service::generate(&state, &payload).await?;
    Ok(ok(serde_json::json!({ "generated": generated })))
}
