pub mod error;
pub mod model;
pub mod schema;

pub use error::Error;

use std::sync::OnceLock;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::{BuildError, Object, Pool, PoolError};
use diesel_async::AsyncPgConnection;

pub type Connection = Object<AsyncPgConnection>;

static POOL: OnceLock<Pool<AsyncPgConnection>> = OnceLock::new();

pub async fn init(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
    database: &str,
) -> Result<(), BuildError> {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        user,
        password,
        host,
        port,
        database,
    );
    let config = AsyncDieselConnectionManager::new(url);
    let pool = Pool::builder(config).build()?;

    POOL.set(pool).map_err(|_| "Pool already initialized").unwrap();

    Ok(())
}

pub async fn get() -> Result<Connection, PoolError> {
    let conn = POOL.get().expect("Pool is not initialized").get().await?;
    Ok(conn)
}
