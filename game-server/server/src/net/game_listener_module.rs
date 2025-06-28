use crate::module::Module;

use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[derive(Debug, Deserialize)]
pub struct GameListenerBlueprint {
    port: u16,
}

pub struct GameListenerModule {
    listener: TcpListener,
}

impl GameListenerModule {
    pub async fn new(blueprint: GameListenerBlueprint) -> Result<Self, Box<dyn Error>> {
        let address = SocketAddr::from(([0, 0, 0, 0], blueprint.port));
        let listener = TcpListener::bind(address).await?;

        Ok(GameListenerModule {
            listener,
        })
    }
}

#[async_trait]
impl Module for GameListenerModule {
    async fn run(&mut self, mut terminate_rx: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
        tokio::select! {
            result = self.listener.accept() => match result {
                Ok((socket, _)) => {
                    todo!()
                },
                Err(e) => return Err(Box::new(e)),
            },
            _ = terminate_rx.recv() => {},
        }

        Ok(())
    }
}