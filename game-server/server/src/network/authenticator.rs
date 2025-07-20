use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use bytes::Bytes;
use crate::network::gateway::{Gateway, NewPlayer};
use crate::player::account::*;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use game_protocol::*;
use game_protocol::auth::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{info, error};

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

#[derive(Debug)]
pub struct Entry {
    pub account_id: i64,
    pub character_id: i64,
    pub privilege: Privilege
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewUnauthorizedSession {
    pub socket: TcpStream,
}

impl Handler<NewUnauthorizedSession> for Authenticator {
    type Result = ();

    fn handle(&mut self, msg: NewUnauthorizedSession, ctx: &mut Self::Context) -> Self::Result {
        const READ_TIMEOUT: Duration = Duration::from_secs(5);

        // Read only one message with timeout
        ctx.spawn(async move {
            let mut socket = msg.socket;

            let mut header_buf = [0u8; PROTOCOL_HEADER_SIZE];
            match timeout(READ_TIMEOUT, socket.read_exact(&mut header_buf)).await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut, "Header read timed out")),
            };

            let (category, length) = match decode_header(&header_buf) {
                Ok(header) => header,
                Err(_) => return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData, "Invalid header")),
            };
            if length == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData, "Invalid body length"));
            }
            if category != ProtocolCategory::Auth {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData, "Invalid protocol category"));
            }

            let mut body_buf = vec![0u8; length];
            match timeout(READ_TIMEOUT, socket.read_exact(&mut body_buf)).await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut, "Body read timed out")),
            };

            Ok((Bytes::from(body_buf), socket))
        }
        .into_actor(self)
        .then(|res, act, ctx| {
            match res {
                Ok((body, socket)) => {
                    act.authenticate(body, socket);
                }
                Err(e) => {
                    error!("Error receiving message from unauthorized session: {}", e);
                }
            }

            actix::fut::ready(())
        }));
    }
}

impl Authenticator {
    fn authenticate(
        &self,
        data: Bytes,
        socket: TcpStream
    ) {
        let protocol = match AuthClientProtocol::decode(data) {
            Ok(p) => p.protocol,
            Err(e) => {
                error!("Failed to decode auth protocol: {}", e);
                return;
            },
        };

        let login = match protocol {
            Some(auth_client_protocol::Protocol::Login(l)) => l,
            _ => {
                error!("Invalid auth protocol");
                return;
            },
        };

        let entry = match validate_login(&login, &self.decoding_key, &self.validation) {
            Ok(entry) => entry,
            Err(e) => {
                error!("Failed to validate login: {}", e);
                return;
            }
        };
        info!("Authenticated: {:?}", entry);

        let login_kind = match login::Kind::try_from(login.kind) {
            Ok(m) => m,
            Err(e) => {
                error!("Invalid login method: {}", e);
                return;
            },
        };

        self.gateway.do_send(NewPlayer { socket, login_kind, entry });
    }
}

fn validate_login(
    login: &Login,
    decoding_key: &DecodingKey,
    validation: &Validation,
) -> Result<(Entry), Box<dyn std::error::Error>> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        aid: String, // account_id
        prv: String, // privilege
    }

    let claims = match jsonwebtoken::decode::<Claims>(
        &login.token,
        decoding_key,
        validation,
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => return Err(Box::new(e)),
    };

    let account_id: i64 = match claims.aid.parse() {
        Ok(id) => id,
        Err(e) => return Err(Box::new(e)),
    };

    let privilege = match Privilege::from_str(&claims.prv) {
        Ok(privilege) => privilege,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(Entry {
        account_id,
        character_id: login.character_id,
        privilege,
    })
}
