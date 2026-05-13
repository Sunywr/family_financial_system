use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthDto {
    pub service: String,
    pub version: String,
    pub environment: String,
    pub database_configured: bool,
    pub database_reachable: bool,
}
