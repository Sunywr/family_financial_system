use crate::{
    common::state::AppState,
    dto::user::{CreateUserRequest, UpdateUserRequest, UserListQuery, UserOptionsDto},
    error::app_error::AppError,
    model::user::User,
    repository::user_repository,
    service::auth_service,
};

pub async fn list(state: &AppState, query: &UserListQuery) -> Result<(Vec<User>, u64), AppError> {
    user_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<User, AppError> {
    user_repository::find_by_id(state.db()?, id).await
}

pub async fn create(state: &AppState, payload: &CreateUserRequest) -> Result<User, AppError> {
    validate_create(payload)?;
    let password_hash = auth_service::hash_password(&payload.password);
    let id = user_repository::create_with_password_hash(
        state.db()?,
        &payload.username,
        &payload.display_name,
        &password_hash,
        &payload.role,
        payload.enabled.unwrap_or(true),
    )
    .await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateUserRequest,
) -> Result<User, AppError> {
    validate_update(payload)?;
    user_repository::update(state.db()?, id, payload).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    user_repository::soft_delete(state.db()?, id).await
}

pub async fn options(state: &AppState) -> Result<Vec<UserOptionsDto>, AppError> {
    let users = user_repository::list_options(state.db()?).await?;
    Ok(users
        .into_iter()
        .map(|user| UserOptionsDto {
            id: user.id,
            display_name: user.display_name,
        })
        .collect())
}

fn validate_create(payload: &CreateUserRequest) -> Result<(), AppError> {
    if payload.username.trim().is_empty() || payload.display_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "username and display_name are required".to_string(),
        ));
    }
    if payload.password.trim().is_empty() {
        return Err(AppError::BadRequest("password is required".to_string()));
    }
    if payload.role.trim().is_empty() {
        return Err(AppError::BadRequest("role is required".to_string()));
    }
    Ok(())
}

fn validate_update(payload: &UpdateUserRequest) -> Result<(), AppError> {
    if payload.display_name.trim().is_empty() || payload.role.trim().is_empty() {
        return Err(AppError::BadRequest(
            "display_name and role are required".to_string(),
        ));
    }
    Ok(())
}
