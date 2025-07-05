use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use bytes::BytesMut;
use crate::network::gateway::{Gateway, NewPlayer};
use crate::player::account::*;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use protocol::*;
use protocol::auth::{*, auth_client_protocol::Protocol};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

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

pub struct Entry {
    pub account_id: u64,
    pub character_id: u64,
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
        let future = async move {
            let mut socket = msg.socket;

            let mut header_buf = [0u8; HEADER_SIZE];
            match timeout(READ_TIMEOUT, socket.read_exact(&mut header_buf)).await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => return Err((e, socket)),
                Err(_) => return Err((
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "Header read timed out"),
                    socket
                )),
            };
            let header = deserialize_header(&header_buf);

            let mut body_buf = BytesMut::with_capacity(header.length);
            match timeout(READ_TIMEOUT, socket.read_exact(&mut body_buf[..header.length])).await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => return Err((e, socket)),
                Err(_) => return Err((
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "Body read timed out"),
                    socket
                )),
            };

            Ok((header, body_buf, socket))
        }
        .into_actor(self)
        .then(|res, act, ctx| {
            match res {
                Ok((header, body, socket)) => {
                    act.authenticate(header, body, socket);
                }
                Err((e, _socket)) => {
                    eprintln!("Error receiving message from unauthorized session: {}", e);
                }
            }

            actix::fut::ready(())
        });

        ctx.spawn(future);
    }
}

impl Authenticator {
    fn authenticate(
        &self,
        header: ProtocolHeader,
        body: BytesMut,
        socket: TcpStream
    ) {
        if header.category != ProtocolCategory::Auth {
            eprintln!("Invalid protocol category: {:?}", header.category);
            return;
        }

        let protocol = match AuthClientProtocol::decode(body) {
            Ok(p) => p.protocol,
            Err(e) => {
                eprintln!("Error decoding auth protocol: {}", e);
                return;
            },
        };

        let login = match protocol {
            Some(Protocol::Login(l)) => l,
            _ => {
                eprintln!("Invalid auth protocol");
                return;
            },
        };

        let entry = match validate_login(&login, &self.decoding_key, &self.validation) {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error validating login: {}", e);
                return;
            }
        };
        println!("Authenticated");

        let login_kind = match login::Kind::try_from(login.kind) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Invalid login method: {}", e);
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
    struct RawClaims {
        aid: String, // account_id
        cid: String, // character_id
        prv: String, // privilege
    }

    let claims = match jsonwebtoken::decode::<RawClaims>(
        &login.token,
        decoding_key,
        validation,
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => return Err(Box::new(e)),
    };

    let account_id: u64 = match claims.aid.parse() {
        Ok(id) => id,
        Err(e) => return Err(Box::new(e)),
    };
    let character_id: u64 = match claims.cid.parse() {
        Ok(id) => id,
        Err(e) => return Err(Box::new(e)),
    };
    let privilege = match Privilege::from_str(&claims.prv) {
        Ok(privilege) => privilege,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(Entry {
        account_id,
        character_id,
        privilege,
    })
}
