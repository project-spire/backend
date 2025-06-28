use actix::{Actor, Context};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

pub struct GameListener {
    port: u16,

    stop_tx: broadcast::Sender<()>,
}

impl GameListener {
    pub fn new(port: u16) -> Self {
        let (stop_tx, _) = broadcast::channel(1);
        GameListener{port, stop_tx}
    }
}

impl Actor for GameListener {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        let address = SocketAddr::from(([0, 0, 0, 0], self.port));
        let mut stop_rx = self.stop_tx.subscribe();

        tokio::spawn(async move {
            let mut listener = match TcpListener::bind(address).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error binding: {}", e);
                    return;
                }
            };

            loop {
                tokio::select! {
                    result = accept(&mut listener) => match result {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Error accepting: {}", e);
                        }
                    },
                    _ = stop_rx.recv() => break,
                }
            }
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        _ = self.stop_tx.send(());
    }
}

async fn accept(listener: &mut TcpListener) -> Result<(), Box<dyn Error>> {
    let (socket, _) = listener.accept().await?;

    Ok(())
}
