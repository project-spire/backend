use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{info, error};
use crate::net::gateway::{Gateway, NewPlayer};
use crate::net::session::Entry;
use crate::protocol::{*, auth::*};

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

            Ok::<(TcpStream, Login), Box<dyn std::error::Error>>((socket, login))
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
    fn validate_login(&self, login: Login) -> Result<(Entry, login::Kind), Box<dyn std::error::Error>> {
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            aid: String, // account_id
        }

        let claims = jsonwebtoken::decode::<Claims>(
            &login.token,
            &self.decoding_key,
            &self.validation,
        )?.claims;

        let account_id = uuid::Uuid::parse_str(&claims.aid)?;
        let character_id = match login.character_id {
            Some(id) => uuid::Uuid::from(id),
            None => return Err("Invalid character id".into()),
        };
        let login_kind = login::Kind::try_from(login.kind)?;

        Ok((Entry { account_id, character_id }, login_kind))
    }
}

async fn receive_login(socket: &mut TcpStream) -> Result<Login, Box<dyn std::error::Error>> {
    let mut header_buf = [0u8; HEADER_SIZE];
    timeout(READ_TIMEOUT, socket.read_exact(&mut header_buf)).await??;

    let header = Header::decode(&header_buf)?;

    let mut body_buf = vec![0u8; header.length];
    timeout(READ_TIMEOUT, socket.read_exact(&mut body_buf)).await??;

    let protocol = Protocol::decode(header.id, body_buf.into())?;
    match protocol {
        Protocol::Login(login) => Ok(login),
        _ => Err("Protocol other than Login is received".into())
    }
}
