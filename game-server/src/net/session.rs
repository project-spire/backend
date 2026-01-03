use crate::config;
use bevy_ecs::prelude::*;
use bytes::{Buf, Bytes, BytesMut};
use protocol::game::{encode, Header, IngressLocalProtocol, Protocol, ProtocolHandler};
use quinn::{Connection, RecvStream, SendStream, WriteError};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncReadExt;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};
use util::rate_limiter::RateLimiter;

pub type EgressProtocol = Bytes;

#[derive(Clone, Copy)]
pub struct Entry {
    pub account_id: i64,
    pub character_id: i64,
}

#[derive(Component, Clone)]
pub struct Session {
    pub entry: Entry,
    inner: Arc<SessionInner>,
}

struct SessionInner {
    connection: Connection,
    ingress_protocol_receiver: crossbeam_channel::Receiver<IngressLocalProtocol>,
    egress_protocol_sender: mpsc::UnboundedSender<EgressProtocol>,

    stop_signal_sender: broadcast::Sender<()>,
    receive_finished: AtomicBool,
    send_finished: AtomicBool,
}

#[derive(Debug, thiserror::Error)]
enum Error {
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
        let (stop_signal_sender, stop_signal_receiver) = broadcast::channel(1);

        let session = Self {
            entry,
            inner: Arc::new(SessionInner {
                connection,
                ingress_protocol_receiver,
                egress_protocol_sender,
                stop_signal_sender,
                receive_finished: AtomicBool::new(false),
                send_finished: AtomicBool::new(false),
            }),
        };

        Self::start_receive(
            session.clone(),
            receive_stream,
            ingress_protocol_sender,
            stop_signal_receiver.resubscribe(),
        );
        Self::start_send(
            session.clone(),
            send_stream,
            egress_protocol_receiver,
            stop_signal_receiver.resubscribe(),
        );

        session
    }

    pub fn stop(&self) {
        _ = self.inner.stop_signal_sender.send(());
    }

    pub fn send(&self, protocol: &(impl prost::Message + Protocol)) {
        let bytes = match encode(protocol) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("{} failed to encode protocol: {}", self, e);
                return;
            }
        };

        _ = self.inner.egress_protocol_sender.send(bytes);
    }

    pub fn send_datagram(&self, protocol: Bytes) {
        if let Err(e) = self.inner.connection.send_datagram(protocol) {
            error!("{} failed to send datagram: {}", self, e);
        }
    }

    pub fn try_iter_ingress_protocols(&self) -> crossbeam_channel::TryIter<'_, IngressLocalProtocol> {
        self.inner.ingress_protocol_receiver.try_iter()
    }

    fn start_receive(
        session: Session,
        stream: RecvStream,
        ingress_protocol_sender: crossbeam_channel::Sender<IngressLocalProtocol>,
        stop_signal_receiver: broadcast::Receiver<()>,
    ) {
        async fn do_receive(
            session: &Session,
            mut stream: RecvStream,
            ingress_protocol_sender: crossbeam_channel::Sender<IngressLocalProtocol>,
            mut stop_signal_receiver: broadcast::Receiver<()>,
        ) -> Result<(), Error> {
            let mut ingress_protocols_limiter = config!(net).ingress.protocols_rate_limit
                .map(|params| RateLimiter::new(params));
            let mut ingress_bytes_limiter = config!(net).ingress.bytes_rate_limit
                .map(|params| RateLimiter::new(params));

            let mut buffer = BytesMut::with_capacity(8 * 1024);

            loop {
                while buffer.len() < Header::size() {
                    tokio::select! {
                        biased;
                        res = stream.read_buf(&mut buffer) => {
                            let n = res?;
                            if n == 0 { return Ok(()); }
                        }
                        Ok(_) = stop_signal_receiver.recv() => {
                            return Ok(());
                        }
                    }
                }

                let header = Header::decode(&mut buffer)?;
                buffer.advance(Header::size());

                while buffer.len() < header.length as usize {
                    tokio::select! {
                        biased;
                        res = stream.read_buf(&mut buffer) => {
                            let n = res?;
                            if n == 0 { return Ok(()); }
                        }
                        Ok(_) = stop_signal_receiver.recv() => {
                            return Ok(());
                        }
                    }
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
                        if let Err(crossbeam_channel::TrySendError::Disconnected(_))
                            = ingress_protocol_sender.try_send(protocol) {
                            break;
                        }
                    }
                    ProtocolHandler::Global => {
                        let protocol = protocol::game::decode_global(header.id, body)?;
                        crate::handler::handle_global(session.clone(), protocol);
                    }
                }
            }

            Ok(())
        }

        tokio::spawn(async move {
            if let Err(e) = do_receive(
                &session,
                stream,
                ingress_protocol_sender,
                stop_signal_receiver,
            ).await {
                error!(error = %e, "receive task failed");
            }

            session.inner.receive_finished.store(true, Ordering::Relaxed);
        });
    }

    fn start_send(
        session: Session,
        stream: SendStream,
        egress_protocol_receiver: mpsc::UnboundedReceiver<EgressProtocol>,
        stop_signal_receiver: broadcast::Receiver<()>,
    ) {
        async fn do_send(
            session: &Session,
            mut stream: SendStream,
            mut egress_protocol_receiver: mpsc::UnboundedReceiver<EgressProtocol>,
            mut stop_signal_receiver: broadcast::Receiver<()>,
        ) -> Result<(), Error> {
            let mut protocols = Vec::with_capacity(16);

            loop {
                tokio::select! {
                    biased;
                    Ok(_) = stop_signal_receiver.recv() => {
                        break;
                    }
                    n = egress_protocol_receiver.recv_many(&mut protocols, 16) => {
                        if n == 0 { break; }
                    }
                }

                for protocol in protocols.drain(..) {
                    stream.write_all(&protocol[..]).await?;
                }
            }

            // Ensure remaining protocols are sent before finishing.
            egress_protocol_receiver.close();
            while let Some(protocol) = egress_protocol_receiver.recv().await {
                stream.write_all(&protocol[..]).await?;
            }

            _ = stream.finish();

            session.inner.send_finished.store(false, Ordering::Relaxed);
            Ok(())
        }

        tokio::spawn(async move {
            if let Err(e) = do_send(
                &session,
                stream,
                egress_protocol_receiver,
                stop_signal_receiver
            ).await {
                error!(error = %e, "send task failed");
            }

            session.inner.send_finished.store(true, Ordering::Relaxed);
        });
    }
}

impl Display for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Session(aid: {}, cid: {})",
            self.entry.account_id,
            self.entry.character_id,
        )
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entry(aid: {}, cid: {})",
            self.account_id,
            self.character_id,
        )
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        cleanup,
    ));
}

fn cleanup(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Session)>,
) {
    for (entity, session) in query.iter_mut() {
        let receive_finished = session.inner.receive_finished.load(Ordering::Relaxed);
        let send_finished = session.inner.send_finished.load(Ordering::Relaxed);

        if !receive_finished && !send_finished {
            continue;
        }

        info!("{} is cleaned up", *session);

        session.stop();
        commands.entity(entity).despawn();
    }
}
