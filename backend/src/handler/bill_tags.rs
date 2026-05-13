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
    dto::bill_tag::{BillTagListQuery, CreateBillTagRequest, UpdateBillTagRequest},
    error::app_error::AppError,
    service::bill_tag_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<BillTagListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = bill_tag_service::list(&state, &query).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateBillTagRequest>,
) -> Result<impl IntoResponse, AppError> {
    let id = bill_tag_service::create(&state, &payload).await?;
    Ok(ok(serde_json::json!({ "id": id })))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateBillTagRequest>,
) -> Result<impl IntoResponse, AppError> {
    bill_tag_service::update(&state, id, &payload).await?;
    Ok(ok(serde_json::json!({ "updated": true })))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    bill_tag_service::delete(&state, id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}
