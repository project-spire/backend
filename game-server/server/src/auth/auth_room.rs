use crate::config::AuthConfig;
use crate::core::room::{handle_room_message, InMessageHandleResult, RoomBuilder, RoomPort};
use crate::server::{ServerContext, ServerMessage};
use crate::net::session::{InMessage, SessionPort};
use crate::player::account::*;
use crate::protocol::*;
use crate::protocol::auth::{*, auth_client_protocol::Protocol};
use jsonwebtoken::{Algorithm, Validation, DecodingKey, decode};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aid: String, // account_id
    cid: String, // character_id
    prv: String, // privilege
}

pub fn run_auth_room(
    server_ctx: Arc<ServerContext>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Arc<RoomPort> {
    let builder = RoomBuilder::new();
    builder.add_in_message_handler(handle_in_message);
    
    let (room_message_tx, mut room_message_rx) = mpsc::channel(16);
    let (in_message_tx, mut in_message_rx) = mpsc::channel(64);

    let ctx = Arc::new(RoomPort::new(room_message_tx, in_message_tx));
    let ctx_handle = ctx.clone();

    tokio::spawn(async move {
        let auth_config = AuthConfig::load();
        let mut room_message_buffer = Vec::with_capacity(16);
        let mut in_message_buffer = Vec::with_capacity(64);

        loop {
            tokio::select! {
                n = in_message_rx.recv_many(&mut in_message_buffer, 64) => {
                    if n == 0 {
                        break;
                    }

                    for in_message in in_message_buffer.drain(0..n) {
                        handle_in_message(&server_ctx, &auth_config, in_message).await;
                    }
                },
                n = room_message_rx.recv_many(&mut room_message_buffer, 16) => {
                    if n == 0 {
                        break;
                    }

                    for room_message in room_message_buffer.drain(0..n) {
                        handle_room_message(&ctx_handle, room_message).await;
                    }
                },
                _ = shutdown_rx.recv() => break,
            }
        }
    });

    ctx
}

async fn handle_in_message(message: &InMessage) -> InMessageHandleResult {
    let (session_ctx, category, data) = message;
    if *category != ProtocolCategory::Auth {
        eprintln!("Protocol category not auth: {:?}", category);
        _ = session_ctx.close_tx.send(());
        return InMessageHandleResult::Break;
    }

    let protocol = AuthClientProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode auth protocol: {}", e);
        _ = session_ctx.close_tx.send(());
        return InMessageHandleResult::Break;
    }

    match protocol.unwrap().protocol {
        Some(Protocol::Login(login)) => {
            handle_login(&server_ctx, auth_config, session_ctx, login).await;
        }
        None => {
            _ = session_ctx.close_tx.send(());
        }
    }
    
    InMessageHandleResult::Break
}

async fn handle_login(
    server_ctx: &Arc<ServerContext>,
    auth_config: &AuthConfig,
    session_ctx: Arc<SessionPort>,
    login: Login,
) {
    //TODO: Initialize DecodingKey once, and reuse
    let claims = match decode::<Claims>(
        &login.token,
        &auth_config.key,
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data.claims,
        Err(e) => {
            eprintln!("Error decoding token({}): {}", &login.token, e);
            session_ctx.close().await;
            return;
        }
    };

    let account_id: u64 = match claims.aid.parse() {
        Ok(id) => id,
        _ => {
            eprintln!("Invalid account id: {}", claims.aid);
            session_ctx.close().await;
            return;
        }
    };
    let character_id: u64 = match claims.cid.parse() {
        Ok(id) => id,
        _ => {
            eprintln!("Invalid character id: {}", claims.cid);
            session_ctx.close().await;
            return;
        }
    };
    let privilege = match Privilege::from_str(claims.prv.as_str()) {
        Err(_) => {
            eprintln!("Invalid privilege: {}", claims.prv);
            session_ctx.close().await;
            return;
        },
        Ok(privilege) => privilege
    };

    println!("Authenticated: {}", session_ctx);

    let account = Account {account_id, privilege};
    _ = server_ctx.message_tx.send(ServerMessage::SessionAuthenticated {
        session_ctx,
        account,
        character_id
    }).await;
}
