use std::sync::OnceLock;
use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions, PgTransaction};

use crate::config::config;

pub type Pool = PgPool;
pub type Error = sqlx::Error;
pub type Transaction<'c> = PgTransaction<'c>;

static POOL: OnceLock<Pool> = OnceLock::new();

pub async fn init() -> Result<(), Error> {
    let conn = format!(
        "postgres://{}:{}@{}:{}/{}",
        config().db_user,
        config().db_password,
        config().db_host,
        config().db_port,
        config().db_name,
    );
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect(&conn)
        .await?;

    POOL.set(pool).unwrap();
    Ok(())
}

pub fn get() -> &'static Pool {
    POOL.get().unwrap()
}
