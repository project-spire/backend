mod auth;
mod config;
mod db;
mod error;
mod context;
mod middleware;

mod protocol {
    pub use protocol::*;
}

mod util {
    pub use util::*;
}

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
use crate::protocol::lobby::dev_auth_server::DevAuthServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Initializing...");
    Config::init()?;
    util::id::Generator::init(config().node_id);

    let db_pool = db::connect().await?;
    let ctx = Context::new(db_pool);
    let authenticator = Authenticator::new();

    // let service = ServiceBuilder::new()
    //     .layer(TraceLayer::new_for_grpc())
    //     .service(DevAuthServer::new(ctx));
    //     // .layer(InterceptorLayer::new(authenticator));

    let addr = format!("[::1]:{}", config().lobby_port).parse()?;
    info!("Serving on {}", addr);
    
    Server::builder()
        .tls_config(load_tls_config()?)?
        .layer(TraceLayer::new_for_grpc())
        .add_service(DevAuthServer::new(ctx.clone()))
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
