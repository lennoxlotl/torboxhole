use eyre::eyre;
use rusqlite::Connection;
use std::path::Path;

mod download;

/// Creates a new SQLite database connection, creates all default tables on
///
/// ### Arguments
/// * `path` - Path to database file
pub fn create_connection<P>(path: P) -> eyre::Result<Connection>
where
    P: AsRef<Path>,
{
    let connection =
        Connection::open("database.db").map_err(|e| eyre!("Unable to open database: {}", e))?;
    download::create_table(&connection)?;
    Ok(connection)
}
