use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Create a PostgreSQL connection pool with standard settings.
///
/// Centralizes pool configuration that was previously inline in `main.rs`.
///
/// # Errors
///
/// Returns `sqlx::Error` if the connection cannot be established.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), sqlx::Error> {
/// let pool = terrasight_server::db::pool::create_pool("postgres://localhost/mydb", 10).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}
