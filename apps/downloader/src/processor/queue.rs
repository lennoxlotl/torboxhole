use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use tbh_database::download::{Download, DownloadState, find_downloads_with_state};
use tbh_torbox::TorBoxApiState;

/// Handles queued downloads until they can be grabbed from torbox.
/// This includes both queueing the downloads and waiting for their completion on the torbox side.
pub async fn process_queue(torbox_api_state: &TorBoxApiState, pool: Pool<SqliteConnectionManager>) {
    match start_queued_downloads(torbox_api_state, pool.clone()).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("FATAL: Unable to start queued downloads: {}", err);
        }
    };
}

async fn start_queued_downloads(
    tor_box_api_state: &TorBoxApiState,
    pool: Pool<SqliteConnectionManager>,
) -> eyre::Result<()> {
    let connection = pool.get()?;
    for download in find_downloads_with_state(&connection, DownloadState::Queued)? {
        let temp_path = create_temp_nzb_file(&download).await?;
        let tb_download = tbh_torbox::usenet::create_download(
            &tor_box_api_state,
            Some(&temp_path),
            None,
            // TODO: Change API to take borrowed string instead
            Some(download.name.to_owned()),
            None,
            -1,
            true,
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
        log::info!("Queued download {} on TorBox", &download.name);
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
