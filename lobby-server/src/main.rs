mod config;
mod error;
mod middleware;
mod service;

use crate::config::config;
use crate::middleware::authenticator::Authenticator;
use crate::service::{characters, dev_auth};
use protocol::lobby::{
    characters_server::CharactersServer,
    dev_auth_server::DevAuthServer,
};
use tonic::service::InterceptorLayer;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower::{Layer, ServiceBuilder};
use tower_http::{
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Initializing...");
    config::init()?;
    util::id::init(config().node_id);

    db::init(
        &config().db_user,
        &config().db_password,
        &config().db_host,
        config().db_port,
        &config().db_name
    ).await?;

    let authenticator = Authenticator::new();

    let authenticated_service = ServiceBuilder::new()
        .layer(InterceptorLayer::new(authenticator))
        .service(CharactersServer::new(characters::Server::new()));

    let addr = format!("[::]:{}", config().port).parse()?;
    info!("Serving on {}", addr);
    
    Server::builder()
        .tls_config(load_tls_config()?)?
        .layer(TraceLayer::new_for_grpc())
        .add_service(DevAuthServer::new(dev_auth::Server::new()))
        // .add_service(SteamAuthServer::new())
        .add_service(authenticated_service)
        .serve(addr)
        .await?;

    Ok(())
}

fn load_tls_config() -> Result<ServerTlsConfig, Box<dyn std::error::Error>> {
    let cert = std::fs::read(&config().tls_cert_file)?;
    let key = std::fs::read(&config().tls_key_file)?;

    let identity = Identity::from_pem(cert, key);
    let tls_config = ServerTlsConfig::new().identity(identity);

    Ok(tls_config)
}
