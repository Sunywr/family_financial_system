use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    common::state::AppState,
    dto::auth::{CaptchaResponse, CurrentUserDto, LoginRequest, LoginResponse},
    error::app_error::AppError,
    repository::user_repository,
};

const CAPTCHA_LENGTH: usize = 4;

pub async fn login(state: &AppState, payload: &LoginRequest) -> Result<LoginResponse, AppError> {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::BadRequest(
            "username and password are required".to_string(),
        ));
    }
    verify_captcha(state, &payload.captcha_id, &payload.captcha_code)?;

    let credential = user_repository::find_credential_by_username(state.db()?, &payload.username)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !credential.enabled {
        return Err(AppError::Unauthorized);
    }
    let verified = verify_password(&credential.password_hash, &payload.password);
    if !verified {
        return Err(AppError::Unauthorized);
    }
    if !credential.password_hash.starts_with("sha256$") {
        let upgraded = hash_password(&payload.password);
        let _ = user_repository::update_password_hash(state.db()?, credential.id, &upgraded).await;
    }

    let user = CurrentUserDto {
        id: credential.id,
        username: credential.username,
        display_name: credential.display_name,
        role: credential.role,
    };

    let token = generate_token(
        user.id,
        &state.settings.auth.jwt_secret,
        state.settings.auth.access_token_ttl_minutes,
    )?;

    Ok(LoginResponse { token, user })
}

pub fn captcha(state: &AppState) -> Result<CaptchaResponse, AppError> {
    let code_id = Uuid::new_v4().to_string();
    let answer = rand::rng()
        .sample_iter(Alphanumeric)
        .take(CAPTCHA_LENGTH)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();
    {
        let mut guard = state
            .captcha_store
            .lock()
            .map_err(AppError::internal_with_log)?;
        guard.insert(code_id.clone(), answer.clone());
        if guard.len() > 300 {
            guard.clear();
        }
    }

    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='140' height='42'>\
            <rect width='100%' height='100%' fill='#f8fafc'/>\
            <text x='70' y='28' font-size='24' font-family='monospace' text-anchor='middle' fill='#0f172a'>{}</text>\
        </svg>",
        answer
    );
    let image_base64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
    Ok(CaptchaResponse {
        code_id,
        image_base64: format!("data:image/svg+xml;base64,{image_base64}"),
    })
}

pub async fn me(state: &AppState, headers: &HeaderMap) -> Result<CurrentUserDto, AppError> {
    let user_id = authenticate_headers(
        headers,
        &state.settings.auth.jwt_secret,
        state.settings.auth.access_token_ttl_minutes,
    )?;
    let user = user_repository::find_by_id(state.db()?, user_id).await?;
    Ok(CurrentUserDto {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    })
}

pub fn logout() -> serde_json::Value {
    serde_json::json!({ "logged_out": true })
}

pub async fn authenticate_request(state: &AppState, headers: &HeaderMap) -> Result<u64, AppError> {
    let user_id = authenticate_headers(
        headers,
        &state.settings.auth.jwt_secret,
        state.settings.auth.access_token_ttl_minutes,
    )?;
    let user = user_repository::find_by_id(state.db()?, user_id).await?;
    if !user.enabled {
        return Err(AppError::Unauthorized);
    }
    Ok(user.id)
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("sha256${:x}", hasher.finalize())
}

pub fn verify_password(stored_password: &str, input_password: &str) -> bool {
    if stored_password.starts_with("sha256$") {
        stored_password == hash_password(input_password)
    } else {
        stored_password == input_password
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    value.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)
}

fn generate_token(user_id: u64, secret: &str, ttl_minutes: i64) -> Result<String, AppError> {
    let expires_at = (Utc::now() + Duration::minutes(ttl_minutes)).timestamp();
    let signature = sign_payload(user_id, expires_at, secret);
    Ok(URL_SAFE_NO_PAD.encode(format!("{user_id}:{expires_at}:{signature}")))
}

fn authenticate_headers(
    headers: &HeaderMap,
    secret: &str,
    ttl_minutes: i64,
) -> Result<u64, AppError> {
    let token = extract_bearer_token(headers)?;
    parse_token(token, secret, ttl_minutes)
}

fn parse_token(token: &str, secret: &str, _ttl_minutes: i64) -> Result<u64, AppError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| AppError::Unauthorized)?;
    let decoded = String::from_utf8(decoded).map_err(|_| AppError::Unauthorized)?;
    let mut parts = decoded.split(':');
    let user_id = parts
        .next()
        .ok_or(AppError::Unauthorized)?
        .parse::<u64>()
        .map_err(|_| AppError::Unauthorized)?;
    let expires_at = parts
        .next()
        .ok_or(AppError::Unauthorized)?
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized)?;
    let signature = parts.next().ok_or(AppError::Unauthorized)?;
    if parts.next().is_some() {
        return Err(AppError::Unauthorized);
    }
    if expires_at < Utc::now().timestamp() {
        return Err(AppError::Unauthorized);
    }
    if sign_payload(user_id, expires_at, secret) != signature {
        return Err(AppError::Unauthorized);
    }
    Ok(user_id)
}

fn sign_payload(user_id: u64, expires_at: i64, secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{user_id}:{expires_at}:{secret}").as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verify_captcha(state: &AppState, captcha_id: &str, captcha_code: &str) -> Result<(), AppError> {
    let input_id = captcha_id.trim();
    let input_code = captcha_code.trim().to_uppercase();
    if input_id.is_empty() || input_code.is_empty() {
        return Err(AppError::BadRequest(
            "captcha_id and captcha_code are required".to_string(),
        ));
    }
    let mut guard = state
        .captcha_store
        .lock()
        .map_err(AppError::internal_with_log)?;
    let expected = guard.remove(input_id).ok_or(AppError::Unauthorized)?;
    if expected != input_code {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
