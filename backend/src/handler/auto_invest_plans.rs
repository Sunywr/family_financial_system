use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};

use crate::{
    common::{
        response::{ok, paged},
        state::AppState,
    },
    dto::auto_invest_plan::{
        AutoInvestPlanListQuery, CreateAutoInvestPlanRequest, UpdateAutoInvestPlanRequest,
    },
    error::app_error::AppError,
    service::{auth_service, auto_invest_plan_service},
};

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AutoInvestPlanListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    let (list, total) = auto_invest_plan_service::list(&state, &query, auth_user_id).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    Ok(ok(auto_invest_plan_service::detail(&state, id, auth_user_id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAutoInvestPlanRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    Ok(ok(
        auto_invest_plan_service::create(&state, &payload, auth_user_id).await?,
    ))
}

pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateAutoInvestPlanRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    Ok(ok(
        auto_invest_plan_service::update(&state, id, &payload, auth_user_id).await?,
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth_service::authenticate_request(&state, &headers).await?;
    auto_invest_plan_service::delete(&state, id, auth_user_id).await?;
    Ok(ok(()))
}
