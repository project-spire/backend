use bevy_ecs::world::World;
use crate::server::ServerContext;
use crate::net::session::{InMessage, OutMessage};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

pub enum RoomMessage {
    SessionEnter(TcpStream),
    Broadcast(OutMessage),
}

#[derive(Clone)]
pub struct RoomPort {
    pub message_tx: mpsc::Sender<RoomMessage>,
    pub in_message_tx: mpsc::Sender<InMessage>,
}

pub trait RoomResource: Any + Send + Sync {}

pub trait RoomEvent: Any + Send + Sync {}

pub struct RoomContext {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    events: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl RoomContext {
    pub fn new() -> Self {
        RoomContext {
            resources: HashMap::new(),
            events: HashMap::new(),
        }
    }

    pub fn insert_resource<T: RoomResource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get_resource<T: RoomResource>(&self) -> &T {
        self.resources.get(&TypeId::of::<T>()).unwrap().as_ref().downcast_ref::<T>().unwrap()
    }

    pub fn get_resource_mut<T: RoomResource>(&mut self) -> &mut T {
        self.resources.get_mut(&TypeId::of::<T>()).unwrap().as_ref().downcast_mut::<T>().unwrap()
    }
}

pub enum MessageHandleResult {
    Break,
    Continue,
    Pass,
}

type InMessageHandler = fn(&InMessage, &RoomContext) -> MessageHandleResult;
type RoomMessageHandler = fn(&RoomMessage, &RoomContext) -> MessageHandleResult;

pub struct RoomBuilder {
    pub in_message_handlers: Vec<InMessageHandler>,
    pub in_message_buffer_size: usize,

    pub room_message_handlers: Vec<RoomMessageHandler>,
    pub room_message_buffer_size: usize,

    pub update_interval: Option<tokio::time::Duration>,
}

impl RoomBuilder {
    pub fn new() -> Self {
        RoomBuilder {
            in_message_handlers: Vec::new(),
            in_message_buffer_size: 0,

            room_message_handlers: Vec::new(),
            room_message_buffer_size: 0,

            update_interval: None,
        }
    }
}

impl Default for RoomBuilder {
    fn default() -> Self {
        let mut builder = RoomBuilder::new();

        builder.in_message_buffer_size = 256;
        builder.room_message_buffer_size = 64;

        builder
    }
}

pub fn run_room(
    builder: RoomBuilder,
    server_ctx: Arc<ServerContext>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> RoomPort {
    let (in_message_tx, mut in_message_rx) = mpsc::channel(builder.in_message_buffer_size);
    if builder.in_message_handlers.is_empty() {
        panic!("InMessage handlers must not be empty!");
    }

    let (room_message_tx, mut room_message_rx) = mpsc::channel(builder.room_message_buffer_size);
    if builder.room_message_handlers.is_empty() {
        panic!("RoomMessage handlers must not be empty!");
    }

    let port = RoomPort { message_tx: room_message_tx, in_message_tx };
    let ctx = RoomContext::new();

    tokio::spawn(async move {
        let mut in_message_buffer = Vec::with_capacity(builder.in_message_buffer_size);
        let mut room_message_buffer = Vec::with_capacity(builder.room_message_buffer_size);

        let mut world = World::default();
        let update_enabled = builder.update_interval.is_some();
        let mut update_timer = tokio::time::interval(
            if update_enabled {
                builder.update_interval.unwrap()
            } else {
                tokio::time::Duration::from_secs(1)
            }
        );

        loop {
            tokio::select! {
                n = in_message_rx.recv_many(&mut in_message_buffer, builder.in_message_buffer_size) => {
                    if n == 0 {
                        break;
                    }

                    for in_message in in_message_buffer.drain(0..n) {
                        handle_in_message(
                            &in_message,
                            &builder.in_message_handlers,
                            &ctx,
                            &server_ctx,
                        ).await;
                    }
                },
                n = room_message_rx.recv_many(&mut room_message_buffer, builder.room_message_buffer_size) => {
                    if n == 0 {
                        break;
                    }

                    for room_message in room_message_buffer.drain(0..n) {
                        handle_room_message(
                            &room_message,
                            &builder.room_message_handlers,
                            &ctx,
                            &server_ctx,
                        ).await;
                    }
                },
                _ = update_timer.tick(), if update_enabled => {
                    update();
                }
                _ = shutdown_rx.recv() => break,
            }
        }
    });

    port
}

async fn handle_in_message(
    message: &InMessage,
    handlers: &Vec<InMessageHandler>,
    ctx: &RoomContext,
    server_ctx: &Arc<ServerContext>,
) {
    let mut handled = false;
    for handler in handlers {
        match handler(&message, ctx) {
            MessageHandleResult::Break => {
                handled = true;
                break;
            },
            MessageHandleResult::Continue => {
                handled = true;
                continue;
            },
            MessageHandleResult::Pass => {
                continue;
            }
        }
    }

    if !handled {
        todo!("Print unhandled message error");
    }
}

async fn handle_room_message(
    message: &RoomMessage,
    handlers: &Vec<RoomMessageHandler>,
    ctx: &RoomContext,
    server_ctx: &Arc<ServerContext>,
) {

}

fn update() {

}
