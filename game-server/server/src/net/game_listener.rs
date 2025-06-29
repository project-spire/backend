use actix::{Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, StreamHandler, WrapFuture};
use crate::net::authenticator::{Authenticator, NewConnection};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;

pub struct GameListener {
    port: u16,
    authenticator_addr: Addr<Authenticator>,
}

impl GameListener {
    pub fn new(port: u16, authenticator_addr: Addr<Authenticator>) -> Self {
        GameListener {
            port,
            authenticator_addr,
        }
    }
}

impl Actor for GameListener {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener_future = TcpListener::bind(listen_addr)
            .into_actor(self)
            .then(move |res, _, ctx| {
                match res {
                    Ok(listener) => {
                        println!("Listening on {}", listen_addr);
                        _ = ctx.add_stream(TcpListenerStream::new(listener))
                    },
                    Err(e) => {
                        eprintln!("Error binding: {}", e);
                        ctx.stop();
                    }
                }
                
                actix::fut::ready(())
            });

        ctx.wait(listener_future);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("Game Listener stopped");
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
                println!("Accepted from {}", socket.peer_addr().unwrap());
                self.authenticator_addr.do_send(NewConnection { socket });
            },
            Err(e) => {
                eprintln!("Error accepting: {}", e);
            }
        }
    }
}
