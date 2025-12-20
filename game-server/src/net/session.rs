use crate::config;
use crate::world::time::Time;
use bevy_ecs::prelude::*;
use bytes::{Buf, Bytes};
use protocol::game::{encode, Header, IngressLocalProtocol, Protocol, ProtocolHandler, ProtocolId};
use protocol::game::net::Ping;
use quinn::{Connection, ReadExactError, RecvStream, SendStream, WriteError};
use std::fmt::{Display, Formatter};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_util::codec::LengthDelimitedCodec;
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
    pub ingress_protocol_receiver: crossbeam_channel::Receiver<IngressLocalProtocol>,
    pub egress_protocol_sender: mpsc::UnboundedSender<EgressProtocol>,

    receive_task: tokio::task::JoinHandle<Result<(), Error>>,
    send_task: tokio::task::JoinHandle<Result<(), Error>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Protocol(#[from] protocol::game::Error),

    // #[error(transparent)]
    // Read(#[from] ReadExactError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Write(#[from] WriteError),

    #[error("Ingress protocols limit error: {0}")]
    IngressProtocolsLimit(util::rate_limiter::Error),

    #[error("Ingress bytes limit error: {0}")]
    IngressBytesLimit(util::rate_limiter::Error),
    
    // #[error("Session not found for entity {0}")]
    // SessionNotFound(Entity),
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

        let receive_task = Self::start_receive(receive_stream, ingress_protocol_sender, entry);
        let send_task = Self::start_send(send_stream, egress_protocol_receiver);

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
        self.receive_task.abort();
        self.send_task.abort();
        self.connection.close(0u32.into(), b"Session closed manually");
    }

    fn start_receive(
        stream: RecvStream,
        ingress_protocol_sender: crossbeam_channel::Sender<IngressLocalProtocol>,
        entry: Entry,
    ) -> tokio::task::JoinHandle<Result<(), Error>> {
        let mut reader = LengthDelimitedCodec::builder()
            .length_field_length(size_of::<u16>())
            .length_field_type::<u16>()
            .length_adjustment(2)
            .new_read(stream);

        let mut ingress_protocols_limiter = config!(net).ingress.protocols_rate_limit
            .map(|params| RateLimiter::new(params));
        let mut ingress_bytes_limiter = config!(net).ingress.bytes_rate_limit
            .map(|params| RateLimiter::new(params));

        tokio::spawn(async move {
            while let Some(frame) = reader.next().await {
                let (id, body) = match frame {
                    Ok(mut frame) => {
                        if frame.len() < 2 {
                            return Err(Error::IO(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid header size"
                            )))
                        }

                        let id = frame.split_to(2).get_u16_ne();
                        (id, frame.freeze())
                    },
                    Err(e) => return Err(e.into()),
                };

                if let Some(limiter) = ingress_protocols_limiter.as_mut() {
                    limiter.check().map_err(Error::IngressProtocolsLimit)?;
                }
                if let Some(limiter) = ingress_bytes_limiter.as_mut() {
                    limiter.check_with_value(body.len() as f32).map_err(Error::IngressBytesLimit)?;
                }

                match protocol::game::protocol_handler(id)? {
                    ProtocolHandler::Local => {
                        let protocol = protocol::game::decode_local(id, body)?;
                        if let Err(crossbeam_channel::TrySendError::Disconnected(_)) = ingress_protocol_sender.try_send(protocol) {
                            break;
                        }
                    }
                    ProtocolHandler::Global => {
                        let protocol = protocol::game::decode_global(id, body)?;
                        crate::handler::handle_global(entry, protocol);
                    }
                }
            }

            // loop {
            //     let (id, data) = receive_protocol(&mut stream).await?;
            //
            // }

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

                for proto in protocols.drain(..) {
                    send_protocol(&mut stream, proto).await?;
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

// async fn receive_protocol(stream: &mut RecvStream) -> Result<(ProtocolId, Bytes), Error> {
//     let mut header_buf = [0u8; Header::size()];
//     stream.read_exact(&mut header_buf).await?;
//     let header = Header::decode(&header_buf)?;
//
//     let mut body_buf = vec![0u8; header.length];
//     stream.read_exact(&mut body_buf).await?;
//
//     Ok((header.id, body_buf.into()))
// }

async fn send_protocol(stream: &mut SendStream, data: Bytes) -> Result<(), Error> {
    stream.write_all(&data[..]).await?;
    Ok(())
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

pub fn send(
    world: &mut World,
    entity: Entity,
    protocol: &(impl prost::Message + Protocol),
) {
    let Ok(bytes) = encode(protocol) else {
        return;
    };
    
    let Some(session) = world.get::<Session>(entity) else {
        return;
    };
    _ = session.egress_protocol_sender.send(bytes);
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(ping);
}

fn ping(query: Query<&Session>, time: Res<Time>) {
    if time.ticks % 10 != 0 {
        return;
    }
    
    let Ok(protocol) = encode(&Ping { timestamp: chrono::Utc::now().timestamp() }) else {
        return;
    };

    for session in query.iter() {
        _ = session.connection.send_datagram(protocol.clone());
    }
}
