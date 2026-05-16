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
    auth_user_id: u64,
) -> Result<(Vec<IntelItem>, u64), AppError> {
    let auth_user = user_repository::find_by_id(state.db()?, auth_user_id).await?;
    let effective_query = IntelListQuery {
        pagination: query.pagination.clone(),
        user_id: if auth_user.username == "admin" {
            query.user_id
        } else {
            Some(auth_user_id)
        },
        status: query.status.clone(),
        keyword: query.keyword.clone(),
    };
    intel_repository::list(state.db()?, &effective_query).await
}

pub async fn detail(state: &AppState, id: u64, auth_user_id: u64) -> Result<IntelItem, AppError> {
    let intel = intel_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, intel.user_id).await?;
    Ok(intel)
}

pub async fn create(
    state: &AppState,
    payload: &CreateIntelRequest,
    auth_user_id: u64,
) -> Result<IntelItem, AppError> {
    validate_create(state, payload).await?;
    ensure_owner_access(state, auth_user_id, payload.user_id).await?;
    let id = intel_repository::create(state.db()?, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateIntelRequest,
    auth_user_id: u64,
) -> Result<IntelItem, AppError> {
    let existing = intel_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
    validate_update(payload)?;
    intel_repository::update(state.db()?, id, payload).await?;
    detail(state, id, auth_user_id).await
}

pub async fn delete(state: &AppState, id: u64, auth_user_id: u64) -> Result<(), AppError> {
    let existing = intel_repository::find_by_id(state.db()?, id).await?;
    ensure_owner_access(state, auth_user_id, existing.user_id).await?;
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
