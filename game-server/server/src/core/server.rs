use crate::auth::auth_room;
use crate::core::config::*;
use crate::core::resource::Resource;
use crate::core::room::{RoomPort, RoomMessage};
use crate::core::session::{run_session, InMessage, OutMessage, SessionPort};
use crate::player::PlayerBundle;
use crate::player::account::*;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub enum ServerMessage {
    Broadcast(OutMessage),
    SessionAuthenticated { session_ctx: SessionPort, account: Account, character_id: u64 },
    SessionClosed(SessionPort),
    RoomTransferBegin { player_bundle: PlayerBundle, target: u64 },
    RoomTransferCommit { player_bundle: PlayerBundle, target: u64 },
}

pub struct ServerContext {
    pub message_tx: mpsc::Sender<ServerMessage>,
}

impl ServerContext {
    pub fn new(message_tx: mpsc::Sender<ServerMessage>) -> ServerContext {
        ServerContext { message_tx }
    }
}

pub struct ServerRunOptions {
    pub dry_run: bool,
}

pub async fn run_server(options: ServerRunOptions) -> Result<(), Box<dyn Error>> {
    let (shutdown_tx, _) = broadcast::channel(1);
    let shutdown_rx_listen = shutdown_tx.subscribe();
    let shutdown_rx_handle = shutdown_tx.subscribe();
    let (message_tx, message_rx) = mpsc::channel(64);

    let ctx = Arc::new(ServerContext::new(message_tx));
    let ctx_listen = ctx.clone();
    let ctx_handle = ctx.clone();
    let auth_room_ctx = auth_room::run_auth_room(ctx.clone(), shutdown_tx.subscribe());
    // let station_room_ctx = station_room::run(ctx.clone(), shutdown_tx.subscribe());
    
    let resource = Arc::new(Resource::load().await);
    let resource_handle = resource.clone();

    if options.dry_run {
        println!("Dry running done!");
        return Ok(());
    }

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        let server_config = ServerConfig::load();
        listen(server_config.game_listen_port, ctx_listen, auth_room_ctx, shutdown_rx_listen).await;
    });
    tasks.spawn(async move {
        handle(ctx_handle, resource_handle, message_rx, shutdown_rx_handle).await;
    });

    while let Some(_) = tasks.join_next().await {}
    shutdown_tx.send(())?;

    Ok(())
}

async fn listen(
    port: u16,
    ctx: Arc<ServerContext>,
    auth_room_ctx: Arc<RoomPort>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(listen_addr).await.unwrap();
    println!("Server listening game at {}", listen_addr);

    loop {
        tokio::select! {
            result = listener.accept() => match result {
                Ok((socket, _)) => {
                    if let Err(e) = socket.set_nodelay(true) {
                        eprintln!("Error setting nodelay: {}", e);
                        return;
                    }

                    _ = auth_room_ctx.message_tx.send(RoomMessage::SessionEnter(socket)).await;
                },
                Err(e) => {
                    eprintln!("Error accepting game connection: {}", e);
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

async fn handle(
    ctx: Arc<ServerContext>,
    resource: Arc<Resource>,
    mut message_rx: mpsc::Receiver<ServerMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut rooms = HashMap::new();
    let mut message_buffer = Vec::with_capacity(64);

    loop {
        tokio::select! {
            n = message_rx.recv_many(&mut message_buffer, 64) => {
                if n == 0 {
                    break;
                }

                for message in message_buffer.drain(0..n) {
                    handle_internal(message, &ctx, &resource, &mut rooms).await;
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

async fn handle_internal(
    message: ServerMessage,
    ctx: &Arc<ServerContext>,
    resource: &Arc<Resource>,
    rooms: &mut HashMap<u64, Arc<RoomPort>>,
) {
    match message {
        ServerMessage::Broadcast(message) =>
            handle_broadcast(rooms, message).await,

        ServerMessage::SessionAuthenticated {session_ctx, account, character_id } =>
            handle_session_authenticated(ctx, resource.clone(), session_ctx, account, character_id).await,

        ServerMessage::SessionClosed(session_ctx) =>
            handle_session_closed(session_ctx).await,

        ServerMessage::RoomTransferBegin { player_bundle, target} =>
            handle_room_transfer_begin(&rooms, player_bundle, target).await,

        ServerMessage::RoomTransferCommit { player_bundle, target} => {},
    }
}

async fn handle_broadcast(
    rooms: &mut HashMap<u64, Arc<RoomPort>>,
    message: OutMessage
) {
    for room in rooms.values() {
        _ = room.message_tx.send(RoomMessage::Broadcast(message.clone())).await;
    }
}

async fn handle_session_authenticated(
    ctx: &Arc<ServerContext>,
    resource: Arc<Resource>,
    session_ctx: Arc<SessionPort>,
    account: Account,
    character_id: u64,
) {
    if !session_ctx.is_open().await {
        return
    }

    let server_ctx = ctx.clone();

    tokio::spawn(async move {
        let session = Session::new(session_ctx);
        let client = match resource.db_client().await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Error getting DB client: {}", e);
                //TODO: Disconnect?
                return
            }
        };

        let player_bundle = match PlayerBundle::load(account, character_id, session, &client).await {
            Ok(player_bundle) => player_bundle,
            Err(e) => {
                eprintln!("Error getting player bundle: {}", e);
                //TODO: Disconnect?
                return
            }
        };

        //TODO: Get the last room
        let last_room = 0;

        _ = server_ctx.message_tx.send(
            ServerMessage::RoomTransferBegin {player_bundle, target: last_room});
    });
}

async fn handle_session_closed(session: Arc<SessionPort>) {
    //TODO: Remove from the current room
}

async fn handle_room_transfer_begin(
    rooms: &HashMap<u64, Arc<RoomPort>>,
    player_bundle: Box<PlayerBundle>,
    target: u64,
) {
    if !player_bundle.session.ctx.is_open().await {
        return
    }

    if !rooms.contains_key(&target) {
        eprintln!("Invalid room transfer: {}, target={}", player_bundle.session.ctx, target);
        return;
    }

    {
        let mut in_message_tx = player_bundle.session.ctx.in_message_tx.write().await;
        *in_message_tx = rooms.get(&target).unwrap().in_message_tx.clone();
    }
}
