use crate::core::room::{handle_room_message, RoomPort};
use crate::core::server::ServerContext;
use crate::core::session::{InMessage, SessionPort};
use crate::protocol::*;
use crate::protocol::net::{*, net_client_protocol::Protocol};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

pub fn run(
    server_ctx: Arc<ServerContext>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Arc<RoomPort> {
    let (room_message_tx, mut room_message_rx) = mpsc::channel(16);
    let (in_message_tx, mut in_message_rx) = mpsc::channel(64);

    let ctx = Arc::new(RoomPort::new(room_message_tx, in_message_tx));
    let ctx_handle = ctx.clone();

    tokio::spawn(async move {
        let mut room_message_buffer = Vec::with_capacity(16);
        let mut in_message_buffer = Vec::with_capacity(64);

        loop {
            tokio::select! {
                n = in_message_rx.recv_many(&mut in_message_buffer, 64) => {
                    if n == 0 {
                        break;
                    }

                    for in_message in in_message_buffer.drain(0..n) {
                        handle_in_message(&server_ctx, in_message).await;
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

async fn handle_in_message(server_ctx: &Arc<ServerContext>, message: InMessage) {
    let (session_ctx, category, data) = message;
    if category != ProtocolCategory::Net {
        eprintln!("Protocol not net: {:?}", category);
        session_ctx.close().await;
        return;
    }

    let protocol = NetClientProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode net protocol: {}", e);
        session_ctx.close().await;
        return;
    }

    match protocol.unwrap().protocol {
        Some(Protocol::RoomTransferReady(ready)) => {
            handle_room_transfer_ready(&server_ctx, session_ctx, ready).await;
        }
        None => {
            _ = session_ctx.close_tx.send(());
        }
        _ => {}
    }
}

async fn handle_room_transfer_ready(
    server_ctx: &Arc<ServerContext>,
    session_ctx: Arc<SessionPort>,
    ready: RoomTransferReady,
) {

}