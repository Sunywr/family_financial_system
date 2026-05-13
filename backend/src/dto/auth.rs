use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_code: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: CurrentUserDto,
}

#[derive(Debug, Serialize)]
pub struct CaptchaResponse {
    pub code_id: String,
    pub image_base64: String,
}

#[derive(Debug, Serialize)]
pub struct CurrentUserDto {
    pub id: u64,
    pub username: String,
    pub display_name: String,
    pub role: String,
}
