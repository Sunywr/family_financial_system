use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sqlx::MySqlPool;

use crate::{config::settings::Settings, error::app_error::AppError};

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: Option<MySqlPool>,
    pub legacy_db: Option<MySqlPool>,
    pub captcha_store: Arc<Mutex<HashMap<String, String>>>,
}

impl AppState {
    pub fn new(settings: Settings, db: Option<MySqlPool>, legacy_db: Option<MySqlPool>) -> Self {
        Self {
            settings: Arc::new(settings),
            db,
            legacy_db,
            captcha_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn db(&self) -> Result<&MySqlPool, AppError> {
        self.db
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("database is not configured".to_string()))
    }

    pub fn legacy_db(&self) -> Result<&MySqlPool, AppError> {
        self.legacy_db
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("legacy database is not configured".to_string()))
    }
}
