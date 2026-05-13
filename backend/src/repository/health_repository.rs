use sqlx::Row;

use crate::{common::state::AppState, error::app_error::AppError};

pub async fn ping_database(state: &AppState) -> Result<bool, AppError> {
    let Some(pool) = &state.db else {
        return Ok(false);
    };

    let row = sqlx::query("SELECT 1 AS ok").fetch_one(pool).await?;
    let value: i32 = row.try_get("ok")?;
    Ok(value == 1)
}
