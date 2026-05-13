use crate::{
    common::state::AppState,
    dto::bill_tag::{BillTagListQuery, CreateBillTagRequest, UpdateBillTagRequest},
    error::app_error::AppError,
    repository::bill_tag_repository,
};

pub async fn list(
    state: &AppState,
    query: &BillTagListQuery,
) -> Result<(Vec<crate::model::bill_tag::BillTag>, u64), AppError> {
    bill_tag_repository::list(state.db()?, query).await
}

pub async fn create(state: &AppState, payload: &CreateBillTagRequest) -> Result<u64, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if bill_tag_repository::find_by_name(state.db()?, payload.user_id, payload.name.trim())
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("tag already exists".to_string()));
    }
    bill_tag_repository::create(state.db()?, payload).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBillTagRequest,
) -> Result<(), AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    bill_tag_repository::update(state.db()?, id, payload).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    bill_tag_repository::soft_delete(state.db()?, id).await
}
