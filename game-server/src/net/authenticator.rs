use actix::prelude::*;
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use quinn::{Connection, RecvStream, SendStream};
use tokio::time::timeout;
use tracing::{error, info};
use crate::config::config;
use crate::net::gateway::{Gateway, NewPlayer};
use crate::net::session::Entry;
use crate::protocol::{*, auth::*};
use crate::util::token;

const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Authenticator {
    decoding_key: DecodingKey,

    gateway: Addr<Gateway>,
}

impl Authenticator {
    pub fn new(gateway: Addr<Gateway>) -> Self {
        let decoding_key = DecodingKey::from_secret(&config().token_key);

        Authenticator { decoding_key, gateway }
    }
}

impl Actor for Authenticator {
    type Context = Context<Self>;
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewUnauthorizedSession {
    pub connection: Connection,
}

impl Handler<NewUnauthorizedSession> for Authenticator {
    type Result = ();

    fn handle(&mut self, msg: NewUnauthorizedSession, ctx: &mut Self::Context) -> Self::Result {
        // Read only one protocol with timeout
        ctx.spawn(async move {
            let connection = msg.connection;
            let mut recv_stream = timeout(READ_TIMEOUT, connection.accept_uni()).await??;

            let login = recv_login(&mut recv_stream).await?;

            Ok::<(Connection, Login), Box<dyn std::error::Error>>((connection, login))
        }
        .into_actor(self)
        .then(|res, act, ctx| {
            let (connection, login) = match res {
                Ok(o) => o,
                Err(e) => {
                    error!("Failed to receive login protocol: {}", e);
                    return actix::fut::ready(());
                }
            };

            let (entry, login_kind) = match act.validate_login(login) {
                Ok(o) => o,
                Err(e) => {
                    error!("Failed to validate login: {}", e);
                    return actix::fut::ready(());
                },
            };

            info!("Authenticated: {:?}", entry);
            act.gateway.do_send(NewPlayer {
                connection,
                login_kind,
                entry
            });

            actix::fut::ready(())
        }));
    }
}

impl Authenticator {
    fn validate_login(&self, login: Login) -> Result<(Entry, login::Kind), Box<dyn std::error::Error>> {
        let claims = token::verify(&login.token, &self.decoding_key)?;
        let entry = Entry {
            account_id: claims.account_id,
            character_id: login.character_id,

        };
        let login_kind = login::Kind::try_from(login.kind)?;

        Ok((entry, login_kind))
    }
}

async fn recv_login(stream: &mut RecvStream) -> Result<Login, Box<dyn std::error::Error>> {
    let mut header_buf = [0u8; HEADER_SIZE];
    timeout(READ_TIMEOUT, stream.read_exact(&mut header_buf)).await??;

    let header = Header::decode(&header_buf)?;

    let mut body_buf = vec![0u8; header.length];
    timeout(READ_TIMEOUT, stream.read_exact(&mut body_buf)).await??;

    let protocol = Protocol::decode(header.id, body_buf.into())?;
    match protocol {
        Protocol::Login(login) => Ok(login),
        _ => Err("Protocol other than Login is received".into())
    }
}
