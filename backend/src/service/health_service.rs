use crate::{
    common::state::AppState, dto::health::HealthDto, error::app_error::AppError,
    repository::health_repository,
};

pub async fn build_health(state: &AppState) -> Result<HealthDto, AppError> {
    let database_configured = !state.settings.database.url.is_empty();
    let database_reachable = if database_configured {
        health_repository::ping_database(state)
            .await
            .unwrap_or(false)
    } else {
        false
    };

    Ok(HealthDto {
        service: state.settings.app.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string()),
        database_configured,
        database_reachable,
    })
}
