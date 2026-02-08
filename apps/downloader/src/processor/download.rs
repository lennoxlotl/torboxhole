use eyre::eyre;
use futures_util::StreamExt;
use log::{error, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use sanitize_filename::sanitize;
use std::fs;
use std::path::{Path, PathBuf};
use rusqlite::ffi::sqlite3changegroup_output;
use tbh_database::download::{Download, DownloadState, find_downloads_with_state, set_download_lock, set_download_state, set_download_retries};
use tbh_torbox::TorBoxApiState;
use tokio::io::AsyncWriteExt;

pub async fn process_download(
    torbox_api_state: &TorBoxApiState,
    pool: Pool<SqliteConnectionManager>,
    download_dir: &PathBuf,
) {
    match download_files(torbox_api_state, pool, download_dir).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("FATAL: Unable to download files: {}", err);
        }
    };
}

async fn download_files(
    torbox_api_state: &TorBoxApiState,
    pool: Pool<SqliteConnectionManager>,
    download_dir: &PathBuf,
) -> eyre::Result<()> {
    let conn = pool.get()?;

    for download in find_downloads_with_state(&conn, DownloadState::Ready)? {
        if let Err(e) =
            handle_download(download.clone(), &torbox_api_state, &pool, &download_dir).await
        {
            error!("Error handling download {}: {}", download.name, e);
            // Allow 5 download retries, evict download otherwise
            if download.retries > 5 {
                set_download_state(&conn, download.id, DownloadState::Failed)?;
                error!(
                    "Download {} exceeded maximum retry limit, evicting download",
                    &download.name
                );
                continue;
            }
            set_download_retries(&conn, download.id, download.retries + 1)?;
            let _ = set_download_lock(&conn, download.id, false);
            set_download_state(&conn, download.id, DownloadState::Ready)?;
        }
    }

    Ok(())
}

async fn handle_download(
    download: Download,
    torbox_api_state: &TorBoxApiState,
    pool: &Pool<SqliteConnectionManager>,
    download_dir: &Path,
) -> eyre::Result<()> {
    let conn = pool.get()?;
    if download.lock {
        return Ok(());
    }
    
    set_download_lock(&conn, download.id, true)?;
    set_download_state(&conn, download.id, DownloadState::Downloading)?;
    info!("Starting download for: {}", download.name);

    let link = match tbh_torbox::usenet::request_dl(
        torbox_api_state,
        download.download_id as i32,
        None,
        true,
        None,
    )
    .await
    {
        Ok(l) => l,
        Err(e) => {
            set_download_lock(&conn, download.id, false)?;
            return Err(eyre!("Failed to request download link: {}", e));
        }
    };

    let file_name = sanitize(&download.name) + ".zip";
    let file_path = download_dir.join(&file_name);
    // Delete any old, failed attempts
    let _ = fs::remove_file(&file_path);

    info!("Using download link: {}", &link);
    if let Err(e) = download_file(&link, &file_path).await {
        set_download_lock(&conn, download.id, false)?;
        return Err(eyre!("Failed to download file: {}", e));
    }

    set_download_state(&conn, download.id, DownloadState::Completed)?;
    set_download_lock(&conn, download.id, false)?;
    info!("Finished downloading: {}", download.name);

    Ok(())
}

async fn download_file(url: &str, path: &Path) -> eyre::Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    // Download request failed
    if !response.status().is_success() {
        return Err(eyre!("Download failed with status: {}", response.status()));
    }

    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
    }

    Ok(())
}
