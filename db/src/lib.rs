pub mod error;

pub use error::Error;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::AsyncPgConnection;
use std::sync::OnceLock;

pub type Connection = Object<AsyncPgConnection>;

static POOL: OnceLock<Pool<AsyncPgConnection>> = OnceLock::new();

pub async fn init(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
    database: &str,
) -> Result<(), Error> {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        user,
        password,
        host,
        port,
        database,
    );
    let config = AsyncDieselConnectionManager::new(url);
    let pool = Pool::builder(config)
        .build()
        .map_err(|e| Error::from(e))?;

    POOL.set(pool).map_err(|_| "Pool already initialized").unwrap();
    _ = conn().await?;

    Ok(())
}

pub async fn conn() -> Result<Connection, Error> {
    POOL.get()
        .unwrap()
        .get()
        .await
        .map_err(|e| Error::from(e))
}
