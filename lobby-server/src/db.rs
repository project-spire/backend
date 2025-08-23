use gel_tokio::{dsn::HostType, Builder, TlsSecurity};
use std::str::FromStr;
use crate::config::Config;

pub type Client = gel_tokio::Client;
pub type Error = gel_errors::Error;

pub async fn connect() -> Result<Client, Box<dyn std::error::Error>> {
    let config = Builder::new()
        .host(HostType::from_str(&Config::get().db_host)?)
        .port(Config::get().db_port)
        .user(&Config::get().db_user)
        .password(&Config::get().db_password)
        .tls_security(TlsSecurity::Insecure)
        .build()?;

    let client = Client::new(&config);
    client.ensure_connected().await?;

    Ok(client)
}
