use gel_tokio::{Builder, Client, TlsSecurity};
use gel_tokio::dsn::HostType;
use std::str::FromStr;
use crate::settings::NetworkSettings;

pub type DbClient = Client;
pub type DbError = gel_errors::Error;

#[derive(Clone)]
pub struct DbContext {
    pub client: DbClient,
}

impl DbContext {
    pub async fn new(settings: &NetworkSettings) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Builder::new()
            .host(HostType::from_str(&settings.db_host)?)
            .port(settings.db_port)
            .user(&settings.db_user)
            .password(&settings.db_password)
            .tls_security(TlsSecurity::Insecure)
            .build()?;

        let client = Client::new(&config);
        client.ensure_connected().await?;

        Ok(DbContext { client })
    }
}
