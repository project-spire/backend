use crate::config;
use bevy_ecs::prelude::*;
use bytes::{Buf, Bytes, BytesMut};
use protocol::game::{encode, Header, IngressLocalProtocol, Protocol, ProtocolHandler};
use quinn::{Connection, RecvStream, SendStream, WriteError};
use std::fmt::{Display, Formatter};
use futures::FutureExt;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tracing::{error, info};
use util::rate_limiter::RateLimiter;

pub type EgressProtocol = Bytes;

#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub account_id: i64,
    pub character_id: i64,
}

#[derive(Component)]
pub struct Session {
    pub entry: Entry,

    pub connection: Connection,
    pub ingress_protocol_receiver: crossbeam_channel::Receiver<(SessionContext, IngressLocalProtocol)>,
    pub egress_protocol_sender: mpsc::UnboundedSender<EgressProtocol>,

    receive_task: Option<tokio::task::JoinHandle<Result<(), Error>>>,
    send_task: Option<tokio::task::JoinHandle<Result<(), Error>>>,
}

#[derive(Clone)]
pub struct SessionContext {
    pub entry: Entry,
    pub egress_protocol_sender: mpsc::UnboundedSender<EgressProtocol>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Protocol(#[from] protocol::game::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Write(#[from] WriteError),

    #[error("Ingress protocols limit error: {0}")]
    IngressProtocolsLimit(util::rate_limiter::Error),

    #[error("Ingress bytes limit error: {0}")]
    IngressBytesLimit(util::rate_limiter::Error),
}

impl Session {
    pub fn start(
        entry: Entry,
        connection: Connection,
        receive_stream: RecvStream,
        send_stream: SendStream,
    ) -> Self {
        let (ingress_protocol_sender, ingress_protocol_receiver) = crossbeam_channel::unbounded();
        let (egress_protocol_sender, egress_protocol_receiver) = mpsc::unbounded_channel();
        let ctx = SessionContext {
            entry: entry.clone(),
            egress_protocol_sender: egress_protocol_sender.clone(),
        };

        let receive_task = Some(Self::start_receive(receive_stream, ingress_protocol_sender, ctx));
        let send_task = Some(Self::start_send(send_stream, egress_protocol_receiver));

        Self {
            entry,
            connection,
            ingress_protocol_receiver,
            egress_protocol_sender,
            receive_task,
            send_task,
        }
    }

    pub fn stop(&self) {
        if let Some(task) = &self.receive_task {
            task.abort();
        }

        if let Some(task) = &self.send_task {
            task.abort();
        }

        self.connection.close(0u32.into(), b"Session closed manually");
    }

    pub fn send(&self, protocol: &(impl prost::Message + Protocol)) {
        let bytes = match encode(protocol) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("{} failed to encode protocol: {}", self, e);
                return;
            }
        };

        _ = self.egress_protocol_sender.send(bytes);
    }

    fn start_receive(
        mut stream: RecvStream,
        ingress_protocol_sender: crossbeam_channel::Sender<(SessionContext, IngressLocalProtocol)>,
        ctx: SessionContext,
    ) -> tokio::task::JoinHandle<Result<(), Error>> {
        let mut ingress_protocols_limiter = config!(net).ingress.protocols_rate_limit
            .map(|params| RateLimiter::new(params));
        let mut ingress_bytes_limiter = config!(net).ingress.bytes_rate_limit
            .map(|params| RateLimiter::new(params));

        tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(8 * 1024);

            loop {
                while buffer.len() < Header::size() {
                    stream.read_buf(&mut buffer).await?;
                }

                let header = Header::decode(&mut buffer)?;
                buffer.advance(Header::size());

                while buffer.len() < header.length as usize {
                    stream.read_buf(&mut buffer).await?;
                }

                let body = buffer.split_to(header.length as usize).freeze();

                if let Some(limiter) = ingress_protocols_limiter.as_mut() {
                    limiter.check().map_err(Error::IngressProtocolsLimit)?;
                }
                if let Some(limiter) = ingress_bytes_limiter.as_mut() {
                    limiter.check_with_value(body.len() as f32).map_err(Error::IngressBytesLimit)?;
                }

                match protocol::game::protocol_handler(header.id)? {
                    ProtocolHandler::Local => {
                        let protocol = protocol::game::decode_local(header.id, body)?;
                        if let Err(crossbeam_channel::TrySendError::Disconnected(_)) = ingress_protocol_sender.try_send((ctx.clone(), protocol)) {
                            break;
                        }
                    }
                    ProtocolHandler::Global => {
                        let protocol = protocol::game::decode_global(header.id, body)?;
                        crate::handler::handle_global(ctx.clone(), protocol);
                    }
                }
            }

            Ok(())
        })
    }

    fn start_send(
        mut stream: SendStream,
        mut egress_protocol_receiver: mpsc::UnboundedReceiver<EgressProtocol>,
    ) -> tokio::task::JoinHandle<Result<(), Error>> {
        tokio::spawn(async move {
            let mut protocols = Vec::with_capacity(16);

            loop {
                egress_protocol_receiver.recv_many(&mut protocols, 16).await;

                for data in protocols.drain(..) {
                    stream.write_all(&data[..]).await?;
                }
            }
        })
    }

    pub fn account_id(&self) -> i64 {
        self.entry.account_id
    }

    pub fn character_id(&self) -> i64 {
        self.entry.character_id
    }
}

impl Display for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Session(account_id: {}, character_id: {})",
            self.entry.account_id, self.entry.character_id,
        )
    }
}

impl SessionContext {
    pub fn send(&self, protocol: &(impl prost::Message + Protocol)) {
        let bytes = match encode(protocol) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("{} failed to encode protocol: {}", self, e);
                return;
            }
        };

        _ = self.egress_protocol_sender.send(bytes);
    }
}

impl Display for SessionContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SessionContext(account_id: {}, character_id: {})",
            self.entry.account_id, self.entry.character_id,
        )
    }
}

// pub fn send(
//     world: &mut World,
//     entity: Entity,
//     protocol: &(impl prost::Message + Protocol),
// ) {
//     let Ok(bytes) = encode(protocol) else {
//         return;
//     };
//
//     let Some(session) = world.get::<Session>(entity) else {
//         return;
//     };
//     _ = session.egress_protocol_sender.send(bytes);
// }

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        cleanup,
    ));
}

fn cleanup(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Session)>,
) {
    for (entity, mut session) in query.iter_mut() {
        let receive_finished = session.receive_task.as_ref().map_or(true, |t| t.is_finished());
        let send_finished = session.send_task.as_ref().map_or(true, |t| t.is_finished());

        if !receive_finished && !send_finished {
            continue;
        }

        if let Some(handle) = session.receive_task.take() {
            if let Some(result) = handle.now_or_never() {
                if let Ok(Err(e)) = result {
                    error!("{} failed to receive: {}", *session, e);
                }
            }
        }

        if let Some(handle) = session.send_task.take() {
            if let Some(result) = handle.now_or_never() {
                if let Ok(Err(e)) = result {
                    error!("{} failed to send: {}", *session, e);
                }
            }
        }

        info!("{} is cleaned up", *session);

        session.stop();
        commands.entity(entity).despawn();
    }
}
