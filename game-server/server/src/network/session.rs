use std::fmt::{Display, Formatter};
use actix::{Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use bevy_ecs::component::Component;
use bytes::Bytes;
use game_protocol::{decode_header, ProtocolCategory, PROTOCOL_HEADER_SIZE};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::error;
use uuid::Uuid;

const EGRESS_PROTOCOL_BUFFER_SIZE: usize = 16;

pub type IngressProtocol = (SessionContext, ProtocolCategory, Bytes);
pub type EgressProtocol = Bytes;

#[derive(Debug, Clone)]
pub struct Entry {
    pub account_id: Uuid,
    pub character_id: Uuid,
}

#[derive(Component, Clone)]
pub struct SessionContext {
    pub entry: Entry,
    pub session: Addr<Session>,
    pub egress_proto_tx: mpsc::Sender<EgressProtocol>,
    pub transfer_tx: mpsc::Sender<mpsc::Sender<IngressProtocol>>,
}

impl SessionContext {
    pub async fn send(&self, proto: EgressProtocol) {
        _ = self.egress_proto_tx.send(proto).await;
    }
    
    pub fn do_send(&self, proto: EgressProtocol) {
        if let Err(e) = self.egress_proto_tx.try_send(proto) {
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
    socket: Option<TcpStream>,

    pub egress_proto_tx: mpsc::Sender<EgressProtocol>,
    egress_proto_rx: Option<mpsc::Receiver<EgressProtocol>>,

    ingress_proto_tx: Option<mpsc::Sender<IngressProtocol>>,

    pub transfer_tx: mpsc::Sender<mpsc::Sender<IngressProtocol>>,
    transfer_rx: Option<mpsc::Receiver<mpsc::Sender<IngressProtocol>>>,
}

impl Session {
    pub fn new(
        entry: Entry,
        socket: TcpStream,
        ingress_proto_tx: mpsc::Sender<IngressProtocol>,
    ) -> Self {
        let (egress_proto_tx, egress_proto_rx) = mpsc::channel(EGRESS_PROTOCOL_BUFFER_SIZE);
        let (transfer_tx, transfer_rx) = mpsc::channel(2);

        Session {
            entry,
            socket: Some(socket),
            egress_proto_tx,
            egress_proto_rx: Some(egress_proto_rx),
            ingress_proto_tx: Some(ingress_proto_tx),
            transfer_tx,
            transfer_rx: Some(transfer_rx),
        }
    }

    fn start_recv(
        &mut self,
        mut reader: ReadHalf<TcpStream>,
        mut ingress_proto_tx: mpsc::Sender<IngressProtocol>,
        mut transfer_rx: mpsc::Receiver<mpsc::Sender<IngressProtocol>>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        let session_ctx = SessionContext {
            entry: self.entry.clone(),
            session: ctx.address(),
            egress_proto_tx: self.egress_proto_tx.clone(),
            transfer_tx: self.transfer_tx.clone(),
        };

        ctx.spawn(
            async move {
                loop {
                    let (category, data) = recv(&mut reader).await?;

                    if let Ok(tx) = transfer_rx.try_recv() {
                        ingress_proto_tx = tx;
                    }
                    _ = ingress_proto_tx.send((session_ctx.clone(), category, data)).await;
                }

                Ok::<(), std::io::Error>(())
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
            }),
        );
    }

    fn start_send(
        &mut self,
        mut writer: WriteHalf<TcpStream>,
        mut egress_proto_rx: mpsc::Receiver<EgressProtocol>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        ctx.spawn(
            async move {
                let mut protos = Vec::with_capacity(EGRESS_PROTOCOL_BUFFER_SIZE);

                loop {
                    egress_proto_rx
                        .recv_many(&mut protos, EGRESS_PROTOCOL_BUFFER_SIZE)
                        .await;

                    for proto in protos.drain(..) {
                        send(&mut writer, proto).await?;
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
        let socket = self.socket.take().expect("Socket must be set before start");
        let (reader, writer) = tokio::io::split(socket);

        let egress_proto_rx = self
            .egress_proto_rx
            .take()
            .expect("Egress protocol channel must be set before start");

        let ingress_proto_tx = self
            .ingress_proto_tx
            .take()
            .expect("Ingress protocol channel must be set before start");

        let transfer_rx = self
            .transfer_rx
            .take()
            .expect("Transfer channel must be set before start");

        self.start_recv(reader, ingress_proto_tx, transfer_rx, ctx);
        self.start_send(writer, egress_proto_rx, ctx);
    }
}

async fn recv(
    reader: &mut ReadHalf<TcpStream>,
) -> Result<(ProtocolCategory, Bytes), std::io::Error> {
    let mut header_buf = [0u8; PROTOCOL_HEADER_SIZE];
    reader.read_exact(&mut header_buf).await?;
    let (category, length) = match decode_header(&header_buf) {
        Ok((c, l)) => (c, l),
        Err(_) => return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData, "Invalid header")),
    };

    if length == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData, "Invalid body length"));
    }

    let mut body_buf = vec![0u8; length];
    reader.read_exact(&mut body_buf).await?;

    Ok((category, Bytes::from(body_buf)))
}

async fn send(writer: &mut WriteHalf<TcpStream>, buffer: Bytes) -> Result<(), std::io::Error> {
    writer.write_all(&buffer[..]).await?;
    Ok(())
}
