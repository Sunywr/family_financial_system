use crate::{
    common::state::AppState,
    dto::intel::{CreateIntelRequest, IntelListQuery, UpdateIntelRequest},
    error::app_error::AppError,
    model::intel_item::IntelItem,
    repository::{intel_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &IntelListQuery,
) -> Result<(Vec<IntelItem>, u64), AppError> {
    intel_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<IntelItem, AppError> {
    intel_repository::find_by_id(state.db()?, id).await
}

pub async fn create(state: &AppState, payload: &CreateIntelRequest) -> Result<IntelItem, AppError> {
    validate_create(state, payload).await?;
    let id = intel_repository::create(state.db()?, payload).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateIntelRequest,
) -> Result<IntelItem, AppError> {
    validate_update(payload)?;
    intel_repository::update(state.db()?, id, payload).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    intel_repository::soft_delete(state.db()?, id).await
}

async fn validate_create(state: &AppState, payload: &CreateIntelRequest) -> Result<(), AppError> {
    let _ = user_repository::find_by_id(state.db()?, payload.user_id).await?;
    validate_common(&payload.title, &payload.status)
}

fn validate_update(payload: &UpdateIntelRequest) -> Result<(), AppError> {
    validate_common(&payload.title, &payload.status)
}

fn validate_common(title: &str, status: &str) -> Result<(), AppError> {
    if title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".to_string()));
    }
    if !matches!(status, "draft" | "active" | "archived") {
        return Err(AppError::BadRequest(
            "status must be draft active or archived".to_string(),
        ));
    }
    Ok(())
}
