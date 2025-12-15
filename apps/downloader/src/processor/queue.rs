use log::{debug, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use tbh_database::download::{count_downloads_with_state, find_downloads_with_state, Download, DownloadState};
use tbh_torbox::TorBoxApiState;

/// Handles queued downloads until they can be grabbed from torbox.
/// This includes both queueing the downloads and waiting for their completion on the torbox side.
pub async fn process_queue(torbox_api_state: &TorBoxApiState, pool: Pool<SqliteConnectionManager>, max_downloads: i32) {
    match start_queued_downloads(torbox_api_state, pool.clone(), max_downloads).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("FATAL: Unable to start queued downloads: {}", err);
        }
    };
}

async fn start_queued_downloads(
    tor_box_api_state: &TorBoxApiState,
    pool: Pool<SqliteConnectionManager>,
    max_downloads: i32,
) -> eyre::Result<()> {
    let connection = pool.get()?;

    // We have no real way of reliably getting this from the TorBox API, this is rather an assumption
    // than being accurate
    let active_downloads = count_downloads_with_state(&connection, DownloadState::Grabbing)?;
    debug!("Currently active downloads: {} (limit: {})", active_downloads, max_downloads);
    if active_downloads >= max_downloads as usize {
        info!("Reached account download limit ({}), will not queue any further downloads", max_downloads);
        return Ok(());
    }

    for download in find_downloads_with_state(&connection, DownloadState::Queued)? {
        let temp_path = create_temp_nzb_file(&download).await?;
        let tb_download = tbh_torbox::usenet::create_download(
            &tor_box_api_state,
            Some(&temp_path),
            None,
            Some(download.name.to_owned()),
            None,
            2,
            false,
            false,
        )
        .await?;

        tbh_database::download::set_download_state(
            &connection,
            download.id,
            DownloadState::Grabbing,
        )?;
        tbh_database::download::set_download_id(
            &connection,
            download.id,
            tb_download.usenetdownload_id as i64,
        )?;

        tokio::fs::remove_file(&temp_path).await?;
        info!("Queued download {} on TorBox", &download.name);
    }
    Ok(())
}

async fn create_temp_nzb_file(download: &Download) -> eyre::Result<PathBuf> {
    let mut path = std::env::temp_dir();
    path.push(format!("tbh_{}.nzb", download.id));
    tokio::fs::write(&path, download.nzb.as_bytes()).await?;
    log::debug!("Wrote temp nzb file to {}", path.to_str().unwrap());
    Ok(path)
}
