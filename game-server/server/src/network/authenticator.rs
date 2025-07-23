use std::error::Error;
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use bytes::Bytes;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use game_protocol::*;
use game_protocol::auth::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{info, error};
use crate::network::gateway::{Gateway, NewPlayer};
use crate::network::session::Entry;
use crate::player::account::*;

const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Authenticator {
    decoding_key: DecodingKey,
    validation: Validation,

    gateway: Addr<Gateway>,
}

impl Authenticator {
    pub fn new(decoding_key: Vec<u8>, gateway: Addr<Gateway>) -> Self {
        let decoding_key = DecodingKey::from_secret(&decoding_key);
        let validation = Validation::new(Algorithm::HS256);

        Authenticator { decoding_key, validation, gateway }
    }
}

impl Actor for Authenticator {
    type Context = Context<Self>;
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewUnauthorizedSession {
    pub socket: TcpStream,
}

impl Handler<NewUnauthorizedSession> for Authenticator {
    type Result = ();

    fn handle(&mut self, msg: NewUnauthorizedSession, ctx: &mut Self::Context) -> Self::Result {
        // Read only one protocol with timeout
        ctx.spawn(async move {
            let mut socket = msg.socket;
            let login = receive_login(&mut socket).await?;

            Ok::<(TcpStream, Login), Box<dyn Error>>((socket, login))
        }
        .into_actor(self)
        .then(|res, act, ctx| {
            let (socket, login) = match res {
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
            act.gateway.do_send(NewPlayer { socket, login_kind, entry });

            actix::fut::ready(())
        }));
    }
}

impl Authenticator {
    fn validate_login(&self, login: Login) -> Result<(Entry, login::Kind), Box<dyn Error>> {
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            aid: String, // account_id
            prv: String, // privilege
        }

        let claims = jsonwebtoken::decode::<Claims>(
            &login.token,
            &self.decoding_key,
            &self.validation,
        )?.claims;

        let account_id: i64 = claims.aid.parse()?;
        let privilege = Privilege::from_str(&claims.prv)?;
        let login_kind = login::Kind::try_from(login.kind)?;

        Ok((Entry {
            account_id,
            character_id: login.character_id,
            privilege,
        }, login_kind))
    }
}

async fn receive_login(socket: &mut TcpStream) -> Result<Login, Box<dyn Error>> {
    let mut header_buf = [0u8; PROTOCOL_HEADER_SIZE];
    timeout(READ_TIMEOUT, socket.read_exact(&mut header_buf)).await??;

    let (category, length) = decode_header(&header_buf)?;
    if length == 0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData, "Invalid body length")));
    }
    if category != ProtocolCategory::Auth {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData, "Invalid protocol category")));
    }

    let mut body_buf = vec![0u8; length];
    timeout(READ_TIMEOUT, socket.read_exact(&mut body_buf)).await??;
    let body_buf = Bytes::from(body_buf);

    let login = match AuthClientProtocol::decode(body_buf)?.protocol {
        Some(auth_client_protocol::Protocol::Login(l)) => l,
        _ => return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData, "Protocol is not Login"))),
    };

    Ok(login)
}
