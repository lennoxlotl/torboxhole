use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::{Path, PathBuf};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReadDirStream;

const FALLBACK_FILE_NAME: &str = "Unknown file name";

/// Handles creation of database rows for all available .nzb files in the ingest directory.
pub async fn process_ingest(ingest_path: String, pool: Pool<SqliteConnectionManager>) {
    let files = match find_nzb_files(ingest_path).await {
        Ok(files) => files,
        Err(err) => {
            log::error!("FATAL: Unable to index files in ingest directory: {}", err);
            return;
        }
    };

    for file in files {
        match create_database_entry(&pool, &file).await {
            Ok(_) => {
                log::debug!(
                    "Ingested File: {:#?}",
                    file.to_str().unwrap_or("Unknown file path")
                );
            }
            Err(_) => {
                log::error!(
                    "FATAL: Unable to create database entry for file: {}",
                    file.to_str().unwrap_or("Unknown file path")
                );
                return;
            }
        };
    }
}

/// Creates a database entry for a singular .nzb file.
async fn create_database_entry(
    pool: &Pool<SqliteConnectionManager>,
    file: &PathBuf,
) -> eyre::Result<()> {
    let connection = pool.get()?;
    let name = file
        .file_name()
        .unwrap_or(FALLBACK_FILE_NAME.as_ref())
        .to_str()
        .unwrap_or(FALLBACK_FILE_NAME)
        .to_string()
        .replace(".nzb", "");
    let content = tokio::fs::read_to_string(file).await?;
    tbh_database::download::create_download(&connection, name, content)?;
    tokio::fs::remove_file(file).await?;
    Ok(())
}

/// Filters for .nzb files in the configurated ingesting directory.
async fn find_nzb_files(ingest_path: impl AsRef<Path>) -> eyre::Result<Vec<PathBuf>> {
    Ok(ReadDirStream::new(tokio::fs::read_dir(ingest_path).await?)
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap())
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>()
        .await)
}
