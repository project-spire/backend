mod config;
mod context;
mod db;
mod error;
mod middleware;
mod service;

use tonic::service::InterceptorLayer;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower::{Layer, ServiceBuilder};
use tower_http::{
    trace::TraceLayer,
};
use tracing::info;

use crate::config::{config, Config};
use crate::context::Context;
use crate::middleware::authenticator::Authenticator;
use protocol::lobby::{
    characters_server::CharactersServer,
    dev_auth_server::DevAuthServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Initializing...");
    Config::init()?;
    util::id::Generator::init(config().node_id);

    let db_pool = db::connect().await?;
    let ctx = Context::new(db_pool);
    let authenticator = Authenticator::new();

    let authenticated_service = ServiceBuilder::new()
        .layer(InterceptorLayer::new(authenticator))
        .service(CharactersServer::new(ctx.clone()));

    let addr = format!("[::]:{}", config().port).parse()?;
    info!("Serving on {}", addr);
    
    Server::builder()
        .tls_config(load_tls_config()?)?
        .layer(TraceLayer::new_for_grpc())
        .add_service(DevAuthServer::new(ctx.clone()))
        // .add_service(SteamAuthServer::new(ctx.clone()))
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
