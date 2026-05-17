use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    common::{response::ok, state::AppState},
    dto::institution::InstitutionCatalogQuery,
    error::app_error::AppError,
    service::institution_catalog_service,
};

pub async fn list(
    State(_state): State<AppState>,
    Query(query): Query<InstitutionCatalogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let data = institution_catalog_service::list(query.investment_type.as_deref()).await?;
    Ok(ok(data))
}
