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
    dto::bill::{BillListQuery, CreateBillRequest, UpdateBillRequest},
    error::app_error::AppError,
    service::bill_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<BillListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = bill_service::list(&state, &query).await?;
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
    Ok(ok(bill_service::detail(&state, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateBillRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(bill_service::create(&state, &payload).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateBillRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(bill_service::update(&state, id, &payload).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    bill_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

pub async fn options() -> Result<impl IntoResponse, AppError> {
    Ok(ok(bill_service::options()))
}
