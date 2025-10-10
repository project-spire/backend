use crate::config::config;
use sqlx::postgres::{PgPool, PgPoolOptions, PgTransaction};
use std::time::Duration;

pub type Pool = PgPool;
pub type Error = sqlx::Error;
pub type Transaction<'c> = PgTransaction<'c>;

pub async fn connect() -> Result<Pool, Error> {
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

    Ok(pool)
}
