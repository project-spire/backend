use crate::settings::NetworkSettings;
use deadpool_postgres::{Config, Runtime};
use deadpool_postgres::tokio_postgres::NoTls;

type Pool = deadpool_postgres::Pool;

pub async fn new_pool(settings: &NetworkSettings) -> Result<Pool, Box<dyn std::error::Error>> {
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
