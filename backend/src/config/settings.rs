use std::net::SocketAddr;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub legacy_database: DatabaseConfig,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("hfs").required(false))
            .add_source(Environment::with_prefix("HFS").separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            host: default_host(),
            port: default_port(),
            log_level: default_log_level(),
        }
    }
}

impl AppConfig {
    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("invalid bind address")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
    #[serde(default = "default_lazy_connect")]
    pub lazy_connect: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            connect_timeout_secs: default_connect_timeout_secs(),
            idle_timeout_secs: default_idle_timeout_secs(),
            acquire_timeout_secs: default_acquire_timeout_secs(),
            lazy_connect: default_lazy_connect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    #[serde(default = "default_scheduler_tick_seconds")]
    pub tick_seconds: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_seconds: default_scheduler_tick_seconds(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_access_token_ttl_minutes")]
    pub access_token_ttl_minutes: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: default_jwt_secret(),
            access_token_ttl_minutes: default_access_token_ttl_minutes(),
        }
    }
}

fn default_app_name() -> String {
    "hfs-backend".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_min_connections() -> u32 {
    0
}

fn default_max_connections() -> u32 {
    8
}

fn default_connect_timeout_secs() -> u64 {
    5
}

fn default_idle_timeout_secs() -> u64 {
    300
}

fn default_acquire_timeout_secs() -> u64 {
    5
}

fn default_lazy_connect() -> bool {
    true
}

fn default_scheduler_tick_seconds() -> u64 {
    30
}

fn default_jwt_secret() -> String {
    "change-me-in-production".to_string()
}

fn default_access_token_ttl_minutes() -> i64 {
    720
}
