use actix::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use quinn::{Endpoint, ServerConfig};
use tracing::{info, error};
use crate::config::Config;
use crate::net::authenticator::{Authenticator, NewUnauthorizedSession};

pub struct GameListener {
    port: u16,
    authenticator: Addr<Authenticator>,
}

impl GameListener {
    pub fn new(authenticator: Addr<Authenticator>) -> Self {
        GameListener {
            port: Config::get().port,
            authenticator,
        }
    }

    fn load_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
        let cert_chain = Config::get_tls_cert_chain()?;
        let private_key = Config::get_tls_key()?;
        let mut server_config = ServerConfig::with_single_cert(cert_chain, private_key)?;

        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into()?));

        Ok(server_config)
    }
}

impl Actor for GameListener {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let server_config = Self::load_server_config().unwrap();
        let endpoint = Endpoint::server(server_config, listen_addr).unwrap();

        let authenticator = self.authenticator.clone();

        info!("Listening on {}", endpoint.local_addr().unwrap());

        ctx.spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                let connection = match incoming.await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to accept: {}", e);
                        continue;
                    }
                };

                info!("Accepted from {}", connection.remote_address());
                authenticator.do_send(NewUnauthorizedSession { connection });
            }
        }.into_actor(self));
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("Game Listener stopped");
    }
}
