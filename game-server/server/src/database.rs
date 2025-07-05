use crate::settings::NetworkSettings;
use deadpool_postgres::{tokio_postgres, Client, Config, Pool, PoolError, Runtime};
use deadpool_postgres::tokio_postgres::NoTls;

pub type DatabaseClient = Client;
pub type DatabaseError = tokio_postgres::Error;
pub type Statement = tokio_postgres::Statement;

pub struct DatabaseContext {
    pool: Pool,
}

impl DatabaseContext {
    pub async fn new(settings: &NetworkSettings) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = new_pool(settings).await?;

        Ok(DatabaseContext { pool })
    }

    pub async fn client(&self) -> Result<DatabaseClient, PoolError> {
        self.pool.get().await
    }
}

async fn new_pool(settings: &NetworkSettings) -> Result<Pool, Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.host = Some(settings.db_host.clone());
    config.port = Some(settings.db_port.clone());
    config.user = Some(settings.db_user.clone());
    config.password = Some(settings.db_password.clone());
    config.dbname = Some(settings.db_name.clone());

    let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)?;
    _ = pool.get().await?; // Check

    Ok(pool)
}
