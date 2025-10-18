use std::net::SocketAddr;
use std::sync::Arc;

use actix::prelude::*;
use quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, ServerConfig};
use tracing::{error, info};

use crate::config::{Config, config};
use crate::net::authenticator::{Authenticator, NewUnauthorizedSession};

pub struct GameListener {
    port: u16,
}

impl GameListener {
    fn load_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
        let cert_chain = Config::get_tls_cert_chain()?;
        let private_key = Config::get_tls_key()?;

        let mut tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;
        tls_config.alpn_protocols = vec![config().application_protocol.as_bytes().to_vec()];

        let mut server_config =
            ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls_config)?));

        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into()?));

        Ok(server_config)
    }
}

impl Default for GameListener {
    fn default() -> Self {
        Self {
            port: config().port,
        }
    }
}

impl Actor for GameListener {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let server_config = Self::load_server_config().unwrap();
        let endpoint = Endpoint::server(server_config, listen_addr).unwrap();

        info!("Listening on {}", endpoint.local_addr().unwrap());

        ctx.spawn(
            async move {
                while let Some(incoming) = endpoint.accept().await {
                    let connection = match incoming.await {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to accept: {}", e);
                            continue;
                        }
                    };

                    info!("Accepted from {}", connection.remote_address());
                    Authenticator::from_registry().do_send(NewUnauthorizedSession { connection });
                }
            }
            .into_actor(self),
        );
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("Game Listener stopped");
    }
}

impl Supervised for GameListener {}

impl SystemService for GameListener {}
