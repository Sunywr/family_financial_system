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
    dto::job::{JobListQuery, JobRunListQuery, TriggerJobRequest, UpdateJobRequest},
    error::app_error::AppError,
    service::job_service,
};

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = job_service::list_jobs(&state, &query).await?;
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
    Ok(ok(job_service::detail_job(&state, id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(job_service::update_job(&state, id, &payload).await?))
}

pub async fn trigger(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<TriggerJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ok(job_service::trigger_job(&state, id, &payload).await?))
}

pub async fn runs(
    State(state): State<AppState>,
    Query(query): Query<JobRunListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total) = job_service::list_runs(&state, &query).await?;
    Ok(paged(
        list,
        total,
        query.pagination.page,
        query.pagination.page_size,
    ))
}
