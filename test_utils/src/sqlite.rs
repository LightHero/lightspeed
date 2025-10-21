use c3p0::{
    sqlx::{Sqlite, sqlite::SqliteConnectOptions},
    *,
};

/// Creates a new connection pool to an in-memory SQLite database.
///
/// The pool is configured to have a maximum lifetime of infinity, an idle timeout of infinity,
/// and a maximum of 1 connection.
///
/// The database is created in memory, and is destroyed when the connection pool is dropped.
pub async fn new_sqlite_db() -> SqliteC3p0Pool {
    let options = SqliteConnectOptions::new().in_memory(true);

    let pool: c3p0::sqlx::Pool<Sqlite> = c3p0::sqlx::pool::PoolOptions::new()
        .max_lifetime(None)
        .idle_timeout(None)
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    SqliteC3p0Pool::new(pool)
}
