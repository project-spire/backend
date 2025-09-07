use actix::prelude::*;
use bevy_ecs::component::Component;
use bytes::Bytes;
use quinn::{Connection, ConnectionError, RecvStream, SendStream};
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc;
use tracing::error;
use crate::protocol::*;

const EGRESS_PROTOCOL_BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub enum Tag {
    Stream,
    Datagram
}

pub type IngressProtocol = (SessionContext, Protocol);
pub type EgressProtocol = Bytes;

#[derive(Debug, Clone)]
pub struct Entry {
    pub account_id: i64,
    pub character_id: i64,
}

#[derive(Component, Clone)]
pub struct SessionContext {
    pub entry: Entry,
    pub session: Addr<Session>,
    pub egress_tx: mpsc::Sender<EgressProtocol>,
    pub transfer_tx: mpsc::Sender<mpsc::Sender<IngressProtocol>>,
}

impl SessionContext {
    pub async fn send(&self, protocol: EgressProtocol) {
        _ = self.egress_tx.send(protocol).await;
    }
    
    pub fn do_send(&self, protocol: EgressProtocol) {
        if let Err(e) = self.egress_tx.try_send(protocol) {
            error!("Error sending egress protocol: {}", e);
        }
    }
}

impl Display for SessionContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Session(account_id: {}, character_id: {})",
            self.entry.account_id,
            self.entry.character_id,
        )
    }
}

pub struct Session {
    entry: Entry,
    connection: Connection,

    pub egress_tx: mpsc::Sender<EgressProtocol>,
    egress_rx: Option<mpsc::Receiver<EgressProtocol>>,

    ingress_tx: Option<mpsc::Sender<IngressProtocol>>,

    pub transfer_tx: mpsc::Sender<mpsc::Sender<IngressProtocol>>,
    transfer_rx: Option<mpsc::Receiver<mpsc::Sender<IngressProtocol>>>,
}

impl Session {
    pub fn new(
        entry: Entry,
        connection: Connection,
        ingress_tx: mpsc::Sender<IngressProtocol>,
    ) -> Self {
        let (egress_tx, egress_rx) = mpsc::channel(EGRESS_PROTOCOL_BUFFER_SIZE);
        let (transfer_tx, transfer_rx) = mpsc::channel(2);

        Session {
            entry,
            connection,
            egress_tx,
            egress_rx: Some(egress_rx),
            ingress_tx: Some(ingress_tx),
            transfer_tx,
            transfer_rx: Some(transfer_rx),
        }
    }

    fn start_recv(
        &mut self,
        mut stream: RecvStream,
        mut ingress_tx: mpsc::Sender<IngressProtocol>,
        mut transfer_rx: mpsc::Receiver<mpsc::Sender<IngressProtocol>>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        let session_ctx = SessionContext {
            entry: self.entry.clone(),
            session: ctx.address(),
            egress_tx: self.egress_tx.clone(),
            transfer_tx: self.transfer_tx.clone(),
        };

        ctx.spawn(async move {
            loop {
                let protocol = recv_from_stream(&mut stream).await?;

                if let Ok(tx) = transfer_rx.try_recv() {
                    ingress_tx = tx;
                }
                _ = ingress_tx.send((session_ctx.clone(), protocol)).await;
            }

            Ok::<(), Box<dyn std::error::Error>>(())
        }
        .into_actor(self)
        .then(|res, _, ctx| {
            match res {
                Ok(_) => {}
                Err(e) => {
                    error!("Error receiving: {}", e);
                }
            }

            ctx.stop();
            actix::fut::ready(())
        }));
    }

    fn start_send(
        &mut self,
        mut stream: SendStream,
        mut egress_rx: mpsc::Receiver<EgressProtocol>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        ctx.spawn(
            async move {
                let mut protocols = Vec::with_capacity(EGRESS_PROTOCOL_BUFFER_SIZE);

                loop {
                    egress_rx
                        .recv_many(&mut protocols, EGRESS_PROTOCOL_BUFFER_SIZE)
                        .await;

                    for proto in protocols.drain(..) {
                        send_to_stream(&mut stream, proto).await?;
                    }
                }

                Ok::<(), std::io::Error>(())
            }
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error sending: {}", e);
                    }
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
        let connection = self.connection.clone();

        let egress_proto_rx = self
            .egress_rx
            .take()
            .expect("Egress protocol channel must be set before start");

        let ingress_proto_tx = self
            .ingress_tx
            .take()
            .expect("Ingress protocol channel must be set before start");

        let transfer_rx = self
            .transfer_rx
            .take()
            .expect("Transfer channel must be set before start");

        ctx.spawn(async move {
            let (send_stream, recv_stream) = connection.accept_bi().await?;
            Ok::<(SendStream, RecvStream), ConnectionError>((
                send_stream,
                recv_stream,
            ))
        }
        .into_actor(self)
        .then(|res, actor, ctx| {
            match res {
                Ok((send_stream, recv_stream)) => {
                    actor.start_send(send_stream, egress_proto_rx, ctx);
                    actor.start_recv(recv_stream, ingress_proto_tx, transfer_rx, ctx);
                },
                Err(e) => {
                    error!("Failed to open bidirectional stream: {}", e);
                    ctx.stop();
                },
            }

            actix::fut::ready(())
        }));

    }
}

async fn recv_from_stream(
    stream: &mut RecvStream,
) -> Result<Protocol, Box<dyn std::error::Error>> {
    let mut header_buf = [0u8; HEADER_SIZE];
    stream.read_exact(&mut header_buf).await?;
    let header = Header::decode(&header_buf)?;

    let mut body_buf = vec![0u8; header.length];
    stream.read_exact(&mut body_buf).await?;

    let protocol = Protocol::decode(header.id, body_buf.into())?;
    Ok(protocol)
}

async fn send_to_stream(stream: &mut SendStream, buffer: Bytes) -> Result<(), std::io::Error> {
    stream.write_all(&buffer[..]).await?;
    Ok(())
}
