mod auth;
mod config;
mod data {
    pub use ::data::*;
}
mod db;
mod lobby_server;
mod protocol {
    pub use protocol::*;
}
mod error;
mod character;

mod util {
    pub use util::*;
}

use tonic::service::InterceptorLayer;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower::ServiceBuilder;
use tracing::info;
use auth::authenticator::Authenticator;
use crate::config::{config, Config};
use crate::lobby_server::LobbyServer;
use crate::protocol::lobby::dev_auth_server::DevAuthServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Initializing...");
    Config::init()?;
    util::id::Generator::init(config().node_id);

    let db_pool = db::connect().await?;
    let lobby_server = LobbyServer::new(db_pool);
    let authenticator = Authenticator::new();

    // let service = ServiceBuilder::new()
    //     // .service(Add some authenticated service here)
    //     .service(DevAuthServer::new(lobby_server))
    //     .layer(InterceptorLayer::new(authenticator));

    let addr = format!("[::1]:{}", config().lobby_port).parse()?;
    info!("Serving on {}", addr);
    
    Server::builder()
        .tls_config(load_tls_config()?)?
        .add_service(DevAuthServer::new(lobby_server))
        .serve(addr)
        .await?;

    Ok(())
}

fn load_tls_config() -> Result<ServerTlsConfig, Box<dyn std::error::Error>> {
    let cert_bytes = std::fs::read(&config().tls_cert_file)?;
    let key_bytes = std::fs::read(&config().tls_key_file)?;

    let identity = Identity::from_pem(cert_bytes, key_bytes);
    let tls_config = ServerTlsConfig::new().identity(identity);

    Ok(tls_config)
}
