use crate::{FromRow, PooledSqliteConn};
use eyre::{OptionExt, eyre};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use rusqlite::Row;
use rusqlite::fallible_iterator::FallibleIterator;

/// Download database model definition
#[derive(Debug, Clone)]
pub struct Download {
    /// Generated download ID
    pub id: i64,
    /// Original .nzb file name for manual investigation
    pub name: String,
    /// Content of .nzb file to be downloaded
    pub nzb: String,
    /// Progress of download on torbox
    pub progress: f32,
    /// Torbox download ID
    pub download_id: i64,
    /// Amount of times the download has been retried (resets on state change)
    pub retries: i64,
    /// If true, the download is currently locked due to file operations in other threads
    pub lock: bool,
    /// State of the download
    pub state: DownloadState,
}

/// Defines the state of a download
#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
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

impl FromRow for Download {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Download {
            id: row.get(0)?,
            name: row.get(1)?,
            nzb: row.get(2)?,
            progress: row.get(3)?,
            download_id: row.get(4)?,
            retries: row.get(5)?,
            lock: row.get::<usize, i32>(6)? == 1,
            state: DownloadState::from_i64(row.get(7)?).unwrap(),
        })
    }
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
                lock INTEGER NOT NULL,
                state INTEGER NOT NULL
            );
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
                VALUES (?1, ?2, 0, 0, 0, 0, 0);
            "#,
            (&name, &nzb),
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to insert download row: {}", e))
}

/// Returns all downloads in a given state.
///
/// ### Arguments
/// * `connection` - Sqlite connection
/// * `state` - Requested download state
pub fn find_downloads_with_state(
    connection: &PooledSqliteConn,
    state: DownloadState,
) -> eyre::Result<Vec<Download>> {
    Ok(connection
        .prepare(
            r#"
            SELECT id, name, nzb, progress, download_id, retries, completed, state
            FROM downloads
            WHERE state = ?1;
        "#,
        )?
        .query([state.to_i32().unwrap()])?
        .map(|row| Download::from_row(row))
        .collect()?)
}

/// Counts all downloads in a given state (returns number of downloads in state).
///
/// ### Arguments
/// * `connection` - Sqlite connection
/// * `state` - Requested download state
pub fn count_downloads_with_state(
    connection: &PooledSqliteConn,
    state: DownloadState,
) -> eyre::Result<usize> {
    connection
        .prepare(
            r#"
            SELECT COUNT(*) FROM downloads WHERE state = ?1;
            "#,
        )?
        .query_one([&state.to_i32().unwrap()], |row| row.get::<usize, usize>(0))
        .map_err(|e| eyre!("Unable to count downloads: {}", e))
}

/// Updates the state of a download row.
///
/// ### Arguments
/// * `connection` - Sqlite connection
/// * `id` - Internal ID of the download
/// * `state` - New download state
pub fn set_download_state(
    connection: &PooledSqliteConn,
    id: i64,
    state: DownloadState,
) -> eyre::Result<()> {
    connection
        .execute(
            r#"
            UPDATE downloads SET retries = 0, state = ?2 WHERE id = ?1;
            "#,
            (id, state.to_i32().unwrap_or_default()),
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to update download state: {}", e))
}

/// Updates the torbox download id.
///
/// ### Arguments
/// * `connection` - Sqlite connection
/// * `id` - Internal ID of the download
/// * `download_id` - New torbox download id
pub fn set_download_id(
    connection: &PooledSqliteConn,
    id: i64,
    download_id: i64,
) -> eyre::Result<()> {
    connection
        .execute(
            r#"
            UPDATE downloads SET download_id = ?2 WHERE id = ?1;
            "#,
            (id, download_id),
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to update download id: {}", e))
}

/// Increments the retries of a download.
///
/// ### Arguments
/// * `connection` - Sqlite connection
/// * `id` - Internal ID of the download
pub fn increment_download_retries(connection: &PooledSqliteConn, id: i64) -> eyre::Result<()> {
    connection
        .execute(
            r#"
            UPDATE downloads SET retries = retries + 1 WHERE id = ?1;
            "#,
            [id],
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to increment download retries: {}", e))
}

/// Sets the lock of a download.
///
/// ### Arguments
/// * `connection` - Sqlite connection
/// * `id` - Internal ID of the download
/// * `lock` - New lock state
pub fn set_download_lock(connection: &PooledSqliteConn, id: i64, lock: bool) -> eyre::Result<()> {
    connection
        .execute(
            r#"
            UPDATE downloads SET lock = ?2 WHERE id = ?1;
            "#,
            (id, if lock { 1 } else { 0 }),
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to update download id: {}", e))
}

/// Resets the locks of all downloads (used on application restart).
/// 
/// ### Arguments
/// * `connection` - Sqlite connection
pub fn reset_locks(connection: &PooledSqliteConn) -> eyre::Result<()> {
    connection
        .execute(
            r#"
            UPDATE downloads SET lock = 0;
            "#,
            [],
        )
        .map(|_| ())
        .map_err(|e| eyre!("Unable to reset download lock: {}", e))
}
