use log::{error, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tbh_database::download::{find_downloads_with_state, increment_download_retries, set_download_id, set_download_state, DownloadState};
use tbh_torbox::TorBoxApiState;
use tbh_torbox::usenet::list;

/// Checks all queued downloads for completion.
pub async fn process_ready_check(tor_box_api_state: &TorBoxApiState, pool: Pool<SqliteConnectionManager>) {
    match check_downloads(tor_box_api_state, pool.clone()).await {
        Ok(_) => {},
        Err(err) => {
            error!("FATAL: Unable to check queued downloads: {}", err);
        }
    }
}

async fn check_downloads(
    tor_box_api_state: &TorBoxApiState,
    pool: Pool<SqliteConnectionManager>,
) -> eyre::Result<()> {
    let connection = pool.get()?;
    for download in find_downloads_with_state(&connection, DownloadState::Grabbing)? {
        let download_state = list(&tor_box_api_state, true, download.download_id as i32).await?;

        // Download has completed, queue for local download
        if download_state.download_finished {
            set_download_state(&connection, download.id, DownloadState::Ready)?;
            info!("Download {} has finished downloading on TorBox, preparing local download", &download.name);
            continue;
        }

        // Allow 1000 seconds (~16 minutes) of downloading / processing, other states get 5 minutes until cancellation
        let in_progress = download_state.download_state == "downloading" || download_state.download_state == "processing";
        let max_retries = if in_progress { 100 } else { 30 };
        if download.retries > max_retries {
            set_download_state(&connection, download.id, DownloadState::Failed)?;
            error!("Download {} exceeded maximum time limit, evicting download", &download.name);
            continue;
        }
        increment_download_retries(&connection, max_retries)?;
    }
    Ok(())
}