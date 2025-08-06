use actix::{Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, StreamHandler, WrapFuture};
use crate::net::authenticator::{Authenticator, NewUnauthorizedSession};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tracing::{info, error};

pub struct GameListener {
    port: u16,
    authenticator: Addr<Authenticator>,
}

impl GameListener {
    pub fn new(port: u16, authenticator: Addr<Authenticator>) -> Self {
        GameListener {
            port,
            authenticator,
        }
    }
}

impl Actor for GameListener {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        ctx.spawn(async move {
            let listener = TcpListener::bind(listen_addr).await?;
            Ok::<TcpListener, std::io::Error>(listener)
        }
        .into_actor(self)
        .then(|res, _, ctx| {
            match res {
                Ok(listener) => {
                    info!("Listening on {}", listener.local_addr().unwrap());
                    _ = ctx.add_stream(TcpListenerStream::new(listener))
                },
                Err(e) => {
                    error!("Error binding: {}", e);
                    ctx.stop();
                }
            }

            actix::fut::ready(())
        }));
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("Game Listener stopped");
    }
}

impl StreamHandler<std::io::Result<TcpStream>> for GameListener {
    fn handle(
        &mut self,
        item: std::io::Result<TcpStream>,
        _: &mut Self::Context,
    ) {
        match item {
            Ok(socket) => {
                info!("Accepted from {}", socket.peer_addr().unwrap());
                self.authenticator.do_send(NewUnauthorizedSession { socket });
            },
            Err(e) => {
                error!("Failed to accept: {}", e);
            }
        }
    }
}
