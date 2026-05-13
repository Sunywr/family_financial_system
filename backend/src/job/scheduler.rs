use std::time::Duration;

use tokio::signal;
use tracing::{error, info};

use crate::{common::state::AppState, service::job_service};

pub async fn run(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    if state.db.is_some() {
        job_service::seed_jobs(&state).await?;
    }
    let tick_seconds = state.settings.scheduler.tick_seconds;
    let mut interval = tokio::time::interval(Duration::from_secs(tick_seconds));

    info!(tick_seconds, "scheduler started");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(error) = job_service::run_due_jobs(&state).await {
                    error!(error = %error, "scheduler tick failed");
                }
            }
            _ = signal::ctrl_c() => {
                info!("scheduler received shutdown signal");
                break;
            }
        }
    }

    Ok(())
}
