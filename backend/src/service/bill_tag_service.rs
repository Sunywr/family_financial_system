use crate::{
    common::state::AppState,
    dto::bill_tag::{
        BillTagListQuery, CreateBillTagRequest, SyncLegacyBillTagsResponse, UpdateBillTagRequest,
    },
    error::app_error::AppError,
    repository::{bill_tag_repository, user_repository},
};

pub async fn list(
    state: &AppState,
    query: &BillTagListQuery,
    _auth_user_id: u64,
) -> Result<(Vec<crate::model::bill_tag::BillTag>, u64), AppError> {
    bill_tag_repository::list(state.db()?, query).await
}

pub async fn create(
    state: &AppState,
    payload: &CreateBillTagRequest,
    auth_user_id: u64,
) -> Result<u64, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if bill_tag_repository::find_by_name(state.db()?, payload.name.trim())
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("tag already exists".to_string()));
    }
    bill_tag_repository::create(
        state.db()?,
        &CreateBillTagRequest {
            user_id: Some(auth_user_id),
            name: payload.name.trim().to_string(),
        },
    )
    .await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBillTagRequest,
    _auth_user_id: u64,
) -> Result<(), AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    bill_tag_repository::update(state.db()?, id, payload).await
}

pub async fn delete(state: &AppState, id: u64, _auth_user_id: u64) -> Result<(), AppError> {
    bill_tag_repository::soft_delete(state.db()?, id).await
}

pub async fn top_tags(
    state: &AppState,
    _auth_user_id: u64,
    category_id: Option<u64>,
) -> Result<Vec<crate::model::bill_tag::BillTag>, AppError> {
    bill_tag_repository::top_tags(state.db()?, category_id).await
}

pub async fn sync_legacy_if_empty(
    state: &AppState,
    user_id: u64,
    auth_user_id: u64,
) -> Result<SyncLegacyBillTagsResponse, AppError> {
    ensure_owner_access(state, auth_user_id, user_id).await?;
    let target_db = state.db()?;
    let existing_total = bill_tag_repository::count_by_user(target_db, user_id).await?;
    if existing_total > 0 {
        return Ok(SyncLegacyBillTagsResponse {
            imported: 0,
            skipped: 0,
            source_total: 0,
            target_total: existing_total,
        });
    }

    let legacy_labels = bill_tag_repository::list_legacy_label_names(state.legacy_db()?).await?;
    let source_total = legacy_labels.len() as u64;
    let mut imported = 0_u64;
    let mut skipped = 0_u64;

    for name in legacy_labels {
        if bill_tag_repository::find_by_name(target_db, &name)
            .await?
            .is_some()
        {
            skipped += 1;
            continue;
        }
        bill_tag_repository::create(
            target_db,
            &CreateBillTagRequest {
                user_id: Some(user_id),
                name,
            },
        )
        .await?;
        imported += 1;
    }

    let target_total = bill_tag_repository::count_by_user(target_db, user_id).await?;
    Ok(SyncLegacyBillTagsResponse {
        imported,
        skipped,
        source_total,
        target_total,
    })
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
