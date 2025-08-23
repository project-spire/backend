mod accountant;
mod authenticator;
mod config;
mod db;
mod lobby_server;
mod token;

mod protocol {
    tonic::include_proto!("spire.protocol.lobby");
}

use tonic::service::InterceptorLayer;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower::ServiceBuilder;
use crate::authenticator::Authenticator;
use crate::config::Config;
use crate::lobby_server::LobbyServer;
use crate::protocol::accountant_server::AccountantServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Config::init()?;

    let db_client = db::connect().await?;
    let lobby_server = LobbyServer::new(db_client);
    let authenticator = Authenticator::new();

    let service = ServiceBuilder::new()
        .layer(InterceptorLayer::new(authenticator))
        .service(AccountantServer::new(lobby_server));
    let addr = format!("[::1]:{}", Config::get().lobby_port).parse()?;
    
    Server::builder()
        .tls_config(load_tls_config()?)?
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}

fn load_tls_config() -> Result<ServerTlsConfig, Box<dyn std::error::Error>> {
    let cert_bytes = std::fs::read(&Config::get().tls_cert_file)?;
    let key_bytes = std::fs::read(&Config::get().tls_key_file)?;

    let identity = Identity::from_pem(cert_bytes, key_bytes);
    let tls_config = ServerTlsConfig::new().identity(identity);

    Ok(tls_config)
}
