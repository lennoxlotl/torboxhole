use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

pub mod download;

pub type PooledSqliteConn = PooledConnection<SqliteConnectionManager>;

pub trait FromRow: Sized {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self>;
}

/// Creates a new SQLite database connection, creates all default tables on
///
/// ### Arguments
/// * `path` - Path to database file
pub fn create_connection<P>(path: P) -> eyre::Result<Pool<SqliteConnectionManager>>
where
    P: AsRef<Path>,
{
    let connection_manager = SqliteConnectionManager::file(path);
    let pool = Pool::new(connection_manager)?;
    download::create_table(&pool.get()?)?;
    Ok(pool)
}
