use std::fmt::{Display, Formatter};
use std::sync::Arc;
use actix::prelude::*;
use bevy_ecs::component::Component;
use bytes::Bytes;
use quinn::{Connection, ConnectionError, ReadExactError, RecvStream, SendStream, WriteError};
use tokio::sync::{mpsc, RwLock, RwLockReadGuard};
use tracing::error;

use crate::handler;
use protocol::game::*;
use crate::world::zone::Zone;

pub type EgressProtocol = Bytes;

#[derive(Debug, Clone)]
pub struct Entry {
    pub account_id: i64,
    pub character_id: i64,
}

pub struct Session {
    entry: Entry,
    connection: Connection,
    addr: Option<Addr<Session>>,

    pub egress_tx: mpsc::UnboundedSender<EgressProtocol>,
    egress_rx: Option<mpsc::UnboundedReceiver<EgressProtocol>>,

    zone_addr: Arc<RwLock<Addr<Zone>>>,
}

#[derive(Component, Clone)]
pub struct SessionContext {
    pub entry: Entry,
    pub egress_tx: mpsc::UnboundedSender<EgressProtocol>,
    pub session_addr: Addr<Session>,
    pub zone_addr: Arc<RwLock<Addr<Zone>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Protocol(#[from] protocol::game::Error),

    #[error(transparent)]
    Read(#[from] ReadExactError),

    #[error(transparent)]
    Write(#[from] WriteError),
}

impl Session {
    pub fn new(
        entry: Entry,
        connection: Connection,
        zone: Addr<Zone>,
    ) -> Self {
        let (egress_tx, egress_rx) = mpsc::unbounded_channel();

        Session {
            entry,
            connection,
            addr: None,
            egress_tx,
            egress_rx: Some(egress_rx),
            zone_addr: Arc::new(RwLock::new(zone)),
        }
    }

    pub fn ctx(&self) -> SessionContext {
        SessionContext {
            entry: self.entry.clone(),
            session_addr: self.addr.clone().unwrap(),
            egress_tx: self.egress_tx.clone(),
            zone_addr: self.zone_addr.clone(),
        }
    }

    pub fn ctx_with_start(self) -> SessionContext {
        let entry = self.entry.clone();
        let egress_tx = self.egress_tx.clone();
        let zone = self.zone_addr.clone();
        let session = self.start();

        SessionContext {
            entry,
            egress_tx,
            zone_addr: zone,
            session_addr: session,
        }
    }

    fn start_recv(
        &mut self,
        mut stream: RecvStream,
        ctx: &mut <Session as Actor>::Context,
    ) {
        let session_ctx = self.ctx();

        ctx.spawn(
            async move {
                loop {
                    let (id, data) = recv_from_stream(&mut stream).await?;
                    handler::decode_and_handle(id, data, &session_ctx).await?;
                }
            }
            .into_actor(self)
            .then(|res: Result<(), Error>, act, ctx| {
                if let Err(e) = res {
                    error!("{} failed to receive: {}", act, e);
                }

                ctx.stop();
                fut::ready(())
            }),
        );
    }

    fn start_send(
        &mut self,
        mut stream: SendStream,
        mut egress_rx: mpsc::UnboundedReceiver<EgressProtocol>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        ctx.spawn(
            async move {
                let mut protocols = Vec::with_capacity(16);

                loop {
                    egress_rx.recv_many(&mut protocols, 16).await;

                    for proto in protocols.drain(..) {
                        send_to_stream(&mut stream, proto).await?;
                    }
                }
            }
            .into_actor(self)
            .then(|res: Result<(), Error>, act, ctx| {
                if let Err(e) = res {
                    error!("{} failed to send: {}", act, e);
                }

                ctx.stop();
                actix::fut::ready(())
            }),
        );
    }
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr = Some(ctx.address());

        let connection = self.connection.clone();

        let egress_rx = self
            .egress_rx
            .take()
            .expect("Egress protocol channel must be set before start");

        ctx.spawn(
            async move {
                let (send_stream, recv_stream) = connection.accept_bi().await?;
                Ok::<(SendStream, RecvStream), ConnectionError>((send_stream, recv_stream))
            }
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok((send_stream, recv_stream)) => {
                        act.start_send(send_stream, egress_rx, ctx);
                        act.start_recv(recv_stream, ctx);
                    }
                    Err(e) => {
                        error!("{} failed to open bidirectional stream: {}", act, e);
                        ctx.stop();
                    }
                }

                fut::ready(())
            }),
        );
    }
}

async fn recv_from_stream(stream: &mut RecvStream) -> Result<(ProtocolId, Bytes), Error> {
    let mut header_buf = [0u8; Header::size()];
    stream.read_exact(&mut header_buf).await?;
    let header = Header::decode(&header_buf)?;

    let mut body_buf = vec![0u8; header.length];
    stream.read_exact(&mut body_buf).await?;

    Ok((header.id, body_buf.into()))
}

async fn send_to_stream(stream: &mut SendStream, buffer: Bytes) -> Result<(), Error> {
    stream.write_all(&buffer[..]).await?;
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

impl SessionContext {
    pub fn account_id(&self) -> i64 {
        self.entry.account_id
    }

    pub fn character_id(&self) -> i64 {
        self.entry.character_id
    }

    pub async fn do_send_to_zone<T>(
        &self,
        msg: T,
    )
    where
        T: actix::Message + Send,
        T::Result: Send,
        Zone: Handler<T>,
        <Zone as Actor>::Context: dev::ToEnvelope<Zone, T>,
    {
        self.zone_addr.read().await.do_send(msg);
    }
}
