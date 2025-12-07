use crate::config;
use crate::net::authenticator::Authenticator;
use crate::net::gateway::{Gateway, NewPlayer};
use actix::prelude::*;
use bytes::Bytes;
use prost::Message;
use protocol::game::auth::Login;
use protocol::game::Header;
use quinn::{Connection, RecvStream, SendStream};
use tokio::time::timeout;
use tracing::{error, info};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewConnection {
    pub connection: Connection,
}

impl Handler<NewConnection> for Authenticator {
    type Result = ();

    fn handle(&mut self, msg: NewConnection, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(async move {
            // Receive login protocol with timeout.
            let connection = msg.connection;
            let (send_stream, mut receive_stream) = timeout(
                config!(app).login.timeout,
                connection.accept_bi(),
            ).await??;

            let login = receive_login(&mut receive_stream).await?;

            Ok::<(Connection, SendStream, RecvStream, Login), Box<dyn std::error::Error>>((
                connection,
                send_stream,
                receive_stream,
                login,
            ))
        }
        .into_actor(self)
        .then(|res, act, _| {
            let (connection, send_stream, receive_stream, login) = match res {
                Ok(o) => o,
                Err(e) => {
                    error!("Failed to receive login protocol: {}", e);
                    return fut::ready(());
                }
            };

            let (entry, login_kind) = match act.validate_login(&login) {
                Ok(o) => o,
                Err(e) => {
                    error!("Failed to validate login: {}", e);
                    return fut::ready(());
                }
            };

            info!("Authenticated: {:?}", entry);
            Gateway::from_registry().do_send(NewPlayer {
                connection,
                receive_stream,
                send_stream,
                login_kind,
                entry,
            });

            fut::ready(())
        }));
    }
}

async fn receive_login(stream: &mut RecvStream) -> Result<Login, Box<dyn std::error::Error>> {
    let mut header_buf = [0u8; Header::size()];
    timeout(config!(app).login.timeout, stream.read_exact(&mut header_buf)).await??;

    let header = Header::decode(&header_buf)?;

    let mut body_buf = vec![0u8; header.length];
    timeout(config!(app).login.timeout, stream.read_exact(&mut body_buf)).await??;

    Ok(Login::decode(Bytes::from(body_buf))?)
}
