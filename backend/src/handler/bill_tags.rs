use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    common::{
        response::{ok, paged},
        state::AppState,
    },
    dto::bill_tag::{
        BillTagListQuery, CreateBillTagRequest, SyncLegacyBillTagsRequest, UpdateBillTagRequest,
    },
    error::app_error::AppError,
    service::{auth_service, bill_tag_service},
};

#[derive(Debug, Deserialize)]
pub struct TopTagsQuery {
    // kept for backward compat; ignored — we use auth_user_id from the token
    pub user_id: Option<u64>,
    pub category_id: Option<u64>,
}

pub async fn top_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TopTagsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let tags = bill_tag_service::top_tags(&state, auth_user_id, query.category_id).await?;
    Ok(ok(tags))
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BillTagListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let (list, total) = bill_tag_service::list(&state, &query, auth_user_id).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateBillTagRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let id = bill_tag_service::create(&state, &payload, auth_user_id).await?;
    Ok(ok(serde_json::json!({ "id": id })))
}

pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateBillTagRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    bill_tag_service::update(&state, id, &payload, auth_user_id).await?;
    Ok(ok(serde_json::json!({ "updated": true })))
}

pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    bill_tag_service::delete(&state, id, auth_user_id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

pub async fn sync_legacy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SyncLegacyBillTagsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let result = bill_tag_service::sync_legacy_if_empty(&state, payload.user_id, auth_user_id).await?;
    Ok(ok(result))
}
