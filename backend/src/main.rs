use std::time::Duration;

use clap::{Parser, Subcommand};
use hfs_backend::{
    common::state::AppState,
    config::settings::Settings,
    job, router,
    service::{bootstrap_service, job_service},
};
use sqlx::mysql::MySqlPoolOptions;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Debug, Parser)]
#[command(name = "hfs-backend")]
#[command(about = "Household Financial System backend")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Serve,
    Scheduler,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let settings = Settings::load()?;
    init_tracing(&settings.app.log_level);
    let state = build_state(settings).await?;

    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(state).await?,
        Command::Scheduler => job::scheduler::run(state).await?,
    }

    Ok(())
}

async fn build_state(settings: Settings) -> Result<AppState, Box<dyn std::error::Error>> {
    let pool = if settings.database.url.is_empty() {
        None
    } else {
        Some(build_pool(&settings.database).await?)
    };

    let legacy_pool = if settings.legacy_database.url.is_empty() {
        None
    } else {
        Some(build_pool(&settings.legacy_database).await?)
    };

    let state = AppState::new(settings, pool, legacy_pool);

    if state.db.is_some() {
        bootstrap_service::seed_builtin_items(&state).await?;
        job_service::seed_jobs(&state).await?;
    }

    Ok(state)
}

async fn build_pool(
    database: &hfs_backend::config::settings::DatabaseConfig,
) -> Result<sqlx::MySqlPool, Box<dyn std::error::Error>> {
    let pool_options = MySqlPoolOptions::new()
        .max_connections(database.max_connections)
        .min_connections(database.min_connections)
        .acquire_timeout(Duration::from_secs(database.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(database.idle_timeout_secs));

    let pool = if database.lazy_connect {
        pool_options.connect_lazy(&database.url)?
    } else {
        pool_options.connect(&database.url).await?
    };

    Ok(pool)
}

async fn serve(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let app = router::create_router(state.clone());
    let bind_addr = state.settings.app.bind_addr();
    let listener = TcpListener::bind(bind_addr).await?;

    info!(address = %bind_addr, "HFS API listening");

    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing(log_level: &str) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("hfs_backend={log_level},tower_http=info").into()),
        )
        .with_target(false)
        .compact()
        .init();
}
