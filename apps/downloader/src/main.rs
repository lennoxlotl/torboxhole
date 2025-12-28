use crate::config::Config;
use eyre::eyre;
use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tbh_torbox::user::UserData;
use tbh_torbox::TorBoxApiState;
use tokio_cron_scheduler::{Job, JobScheduler};
use tbh_database::download::reset_locks;

mod config;
mod processor;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();

    let config: Config = tbh_config::load_config_at(
        "config.toml",
        include_str!("../config.default.toml").to_string(),
        true,
    )?;
    let database = tbh_database::create_connection(config.database_path())?;
    reset_download_locks(&database)?;

    let data = check_torbox_validity(&config.torbox().into()).await?;
    run_jobs(tbh_torbox::download_limit::resolve_download_limit(&data), config, database).await
}

/// Resets all download locks (done on file operations)
fn reset_download_locks(database: &Pool<SqliteConnectionManager>) -> eyre::Result<()> {
    let connection = database.get()?;
    reset_locks(&connection)?;
    Ok(())
}

/// Runs all processor jobs until the process is (un)gracefully terminated.
async fn run_jobs(max_downloads: i32, config: Config, database: Pool<SqliteConnectionManager>) -> eyre::Result<()> {
    let mut scheduler = JobScheduler::new().await?;
    scheduler
        .add(
            // Runs all processor jobs every 10 seconds
            Job::new_async("1/10 * * * * *", move |_, _| {
                let config_clone = config.clone();
                let database_clone = database.clone();
                Box::pin(async move {
                    processor::ingest::process_ingest(
                        config_clone.directories().ingest().to_string(),
                        database_clone.clone(),
                    )
                    .await;
                    processor::queue::process_queue(
                        &config_clone.torbox().into(),
                        database_clone.clone(),
                        max_downloads,
                    )
                    .await;
                    processor::check_ready::process_ready_check(
                        &config_clone.torbox().into(),
                        database_clone.clone(),
                    ).await;
                })
            })?,
        )
        .await?;
    scheduler.start().await?;
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    scheduler.shutdown().await?;
    Ok(())
}

/// Checks TorBox API configuration by fetching the user profile.
async fn check_torbox_validity(config: &TorBoxApiState) -> eyre::Result<UserData> {
    info!("Checking provided TorBox API details...");
    let data = tbh_torbox::user::me(&config, false)
        .await
        .map_err(|_| eyre!("Invalid TorBox API key provided, please check your configuration"))?;
    info!("API details are valid");
    info!(
        "Account has {} available download slots",
        tbh_torbox::download_limit::resolve_download_limit(&data)
    );
    Ok(data)
}
