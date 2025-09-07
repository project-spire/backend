use sqlx::postgres::{PgPool, PgPoolOptions, PgTransaction};
use crate::config::config;

pub type Pool = PgPool;
pub type Error = sqlx::Error;
pub type Transaction<'c> = PgTransaction<'c>;

pub async fn connect() -> Result<Pool, Error> {
    let conn = format!("postgres://{}:{}@{}:{}/{}",
       config().db_user,
       config().db_password,
       config().db_host,
       config().db_port,
       config().db_name,
    );
    let pool = PgPoolOptions::new()
        .connect(&conn)
        .await?;

    Ok(pool)
}