use actix::{Actor, ActorContext, ActorFutureExt, AsyncContext, Context, Handler, WrapFuture};
use bytes::Bytes;
use protocol::{deserialize_header, ProtocolCategory, HEADER_SIZE};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

const EGRESS_MESSAGE_BUFFER_SIZE: usize = 16;

pub type IngressMessage = (SessionContext, ProtocolCategory, Bytes);
pub type EgressMessage = Bytes;

#[derive(Clone)]
pub struct SessionContext {
    egress_msg_tx: mpsc::Sender<EgressMessage>,
}

impl SessionContext {
    pub async fn send(&mut self, msg: EgressMessage) {
        _ = self.egress_msg_tx.send(msg).await;
    }
}

pub struct Session {
    socket: Option<TcpStream>,

    egress_msg_tx: mpsc::Sender<EgressMessage>,
    egress_msg_rx: Option<mpsc::Receiver<EgressMessage>>,

    ingress_msg_tx: Option<mpsc::Sender<IngressMessage>>,
    transfer_tx: mpsc::Sender<mpsc::Sender<IngressMessage>>,
    transfer_rx: Option<mpsc::Receiver<mpsc::Sender<IngressMessage>>>,
}

impl Session {
    pub fn new(socket: TcpStream, ingress_msg_tx: mpsc::Sender<IngressMessage>) -> Self {
        let (egress_msg_tx, egress_msg_rx) = mpsc::channel(EGRESS_MESSAGE_BUFFER_SIZE);
        let (transfer_tx, transfer_rx) = mpsc::channel(1);

        Session {
            socket: Some(socket),
            egress_msg_tx,
            egress_msg_rx: Some(egress_msg_rx),
            ingress_msg_tx: Some(ingress_msg_tx),
            transfer_tx,
            transfer_rx: Some(transfer_rx),
        }
    }
    
    pub fn new_ctx(&mut self) -> SessionContext {
        SessionContext {
            egress_msg_tx: self.egress_msg_tx.clone(),
        }
    }

    fn start_recv(
        &mut self,
        mut reader: ReadHalf<TcpStream>,
        mut ingress_msg_tx: mpsc::Sender<IngressMessage>,
        mut transfer_rx: mpsc::Receiver<mpsc::Sender<IngressMessage>>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        let session_ctx = self.new_ctx();
        ctx.spawn(
            async move {
                loop {
                    let (category, body) = recv(&mut reader).await?;

                    if let Ok(tx) = transfer_rx.try_recv() {
                        ingress_msg_tx = tx;
                    }
                    _ = ingress_msg_tx.send((session_ctx.clone(), category, body)).await;
                }

                Ok::<(), std::io::Error>(())
            }
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error receiving: {}", e);
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
        mut egress_msg_rx: mpsc::Receiver<EgressMessage>,
        ctx: &mut <Session as Actor>::Context,
    ) {
        ctx.spawn(
            async move {
                let mut egress_msg_buf = Vec::with_capacity(EGRESS_MESSAGE_BUFFER_SIZE);

                loop {
                    egress_msg_rx
                        .recv_many(&mut egress_msg_buf, EGRESS_MESSAGE_BUFFER_SIZE)
                        .await;

                    for msg in egress_msg_buf.drain(..) {
                        send(&mut writer, msg).await?;
                    }
                }

                Ok::<(), std::io::Error>(())
            }
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error sending: {}", e);
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

        let egress_msg_rx = self
            .egress_msg_rx
            .take()
            .expect("out-message channel must be set before start");

        let ingress_msg_tx = self
            .ingress_msg_tx
            .take()
            .expect("in-message channel must be set before start");

        let transfer_rx = self
            .transfer_rx
            .take()
            .expect("transfer channel must be set before start");

        self.start_recv(reader, ingress_msg_tx, transfer_rx, ctx);
        self.start_send(writer, egress_msg_rx, ctx);
    }
}

async fn recv(
    reader: &mut ReadHalf<TcpStream>,
) -> Result<(ProtocolCategory, Bytes), std::io::Error> {
    let mut header_buf = [0u8; HEADER_SIZE];
    reader.read_exact(&mut header_buf).await?;
    let header = deserialize_header(&header_buf);

    if header.category == ProtocolCategory::None || header.length == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData, "Invalid header"));
    }

    let mut body_buf = vec![0u8; header.length];
    reader.read_exact(&mut body_buf).await?;

    Ok((header.category, Bytes::from(body_buf)))
}

async fn send(writer: &mut WriteHalf<TcpStream>, buffer: Bytes) -> Result<(), std::io::Error> {
    writer.write_all(&buffer[..]).await?;
    Ok(())
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Transfer {
    ingress_msg_tx: mpsc::Sender<IngressMessage>,
}

impl Handler<Transfer> for Session {
    type Result = ();

    fn handle(&mut self, msg: Transfer, ctx: &mut Self::Context) -> Self::Result {
        let transfer_tx = self.transfer_tx.clone();

        ctx.spawn(async move {
            _ = transfer_tx.send(msg.ingress_msg_tx).await;
        }.into_actor(self));
    }
}
