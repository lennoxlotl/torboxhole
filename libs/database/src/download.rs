use crate::PooledSqliteConn;
use eyre::eyre;

/// Download database model definition
pub struct Download {
    /// Generated download ID
    id: i64,
    /// Original .nzb file name for manual investigation
    name: String,
    /// Content of .nzb file to be downloaded
    nzb: String,
    /// Progress of download on torbox
    progress: f32,
    /// Torbox download ID
    download_id: i64,
    /// Amount of times the download has been retried (resets on state change)
    retries: i64,
    /// If true, the download has either been completed successfully or failed (state shows status)
    completed: bool,
    /// State of the download
    state: DownloadState,
}

/// Defines the state of a download
pub enum DownloadState {
    /// Download is queued
    Queued = 0,
    /// Download is currently being prepared by torbox
    Grabbing = 1,
    /// File is ready to be downloaded from torbox
    Ready = 2,
    /// File is currently downloading to the target directory
    Downloading = 3,
    /// File has been downloaded successfully
    Completed = 4,
    /// The download has failed on local or torbox's end, will be retried (max. 5 times)
    Failed = 5,
    /// Files are currently being extracted from the zip file
    Extracting = 6,
    /// Files have been extracted and the download is complete
    Extracted = 7,
}

/// Creates the table for [`Download`] entities.
pub fn create_table(connection: &PooledSqliteConn) -> eyre::Result<()> {
    connection
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                nzb BLOB NOT NULL,
                progress REAL NOT NULL,
                download_id INTEGER NOT NULL,
                retries INTEGER NOT NULL,
                completed INTEGER NOT NULL,
                state INTEGER NOT NULL
            )
        "#,
            (),
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to create table: {}", e))
}

/// Creates a new download row in the database.
/// Values not represented in arguments will be set to default values.
///
/// ### Arguments
/// * `connection` - Sqlite Connection
/// * `name` - Name of download (usually name of .nzb file)
/// * `nzb` - Content of .nzb file
pub fn create_download(
    connection: &PooledSqliteConn,
    name: String,
    nzb: String,
) -> eyre::Result<()> {
    connection
        .execute(
            r#"
                INSERT INTO downloads (name, nzb, progress, download_id, retries, completed, state)
                VALUES (?1, ?2, 0, 0, 0, 0, 0)
            "#,
            (&name, &nzb),
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to insert download row: {}", e))
}
