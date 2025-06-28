use crate::core::config::DatabaseConfig;
use deadpool_postgres::{Config, Client, Pool, PoolError, Runtime};
use tokio_postgres::NoTls;

pub struct Resource {
    pub db_pool: Pool
}

impl Resource {
    pub async fn load() -> Resource {
        let database_config = DatabaseConfig::load();
        let mut pool_config = Config::new();
        pool_config.host = Some(database_config.host);
        pool_config.port = Some(database_config.port);
        pool_config.user = Some(database_config.user);
        pool_config.password = Some(database_config.password);
        pool_config.dbname = Some(database_config.database);

        let db_pool = pool_config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        _ = db_pool.get().await.unwrap(); // Connection check

        Resource {
            db_pool
        }
    }

    pub async fn db_client(&self) -> Result<Client, PoolError> {
        self.db_pool.get().await
    }
}
