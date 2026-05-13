pub fn parse_u64_id(value: u64) -> Result<i64, crate::error::app_error::AppError> {
    i64::try_from(value)
        .map_err(|_| crate::error::app_error::AppError::BadRequest("id is too large".to_string()))
}
