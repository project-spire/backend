use gel_tokio::{Builder, Client, TlsSecurity};
use gel_tokio::dsn::HostType;
use std::str::FromStr;
use crate::config::Config;

pub type DbClient = Client;
pub type DbError = gel_errors::Error;

#[derive(Clone)]
pub struct DbContext {
    pub client: DbClient,
}

impl DbContext {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Builder::new()
            .host(HostType::from_str(&Config::get().db_host)?)
            .port(Config::get().db_port)
            .user(&Config::get().db_user)
            .password(&Config::get().db_password)
            .tls_security(TlsSecurity::Insecure)
            .build()?;

        let client = Client::new(&config);
        client.ensure_connected().await?;

        Ok(DbContext { client })
    }
}
