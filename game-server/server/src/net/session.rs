use crate::world::zone::Zone;
use actix::{Actor, AsyncContext};
use bevy_ecs::component::Component;
use bytes::{Bytes, BytesMut};
use protocol::{deserialize_header, ProtocolCategory, HEADER_SIZE};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};

const EGRESS_MESSAGE_BUFFER_SIZE: usize = 16;

pub type IngressMessage = (SessionContext, ProtocolCategory, Bytes);
pub type EgressMessage = Bytes;

#[derive(Clone, Component)]
pub struct SessionContext {
    egress_msg_tx: mpsc::Sender<EgressMessage>,
}

impl SessionContext {
    pub async fn send(&mut self, msg: EgressMessage) {
        _ = self.egress_msg_tx.send(msg).await;
    }
}

pub struct SessionRetriever {
    retrieve_tx: broadcast::Sender<()>,
    retrieve_rx: broadcast::Receiver<()>,

    egress_msg_tx: Option<oneshot::Sender<mpsc::Sender<EgressMessage>>>,
    egress_msg_rx: oneshot::Receiver<mpsc::Receiver<EgressMessage>>,

    reader_tx: Option<oneshot::Sender<ReadHalf<TcpStream>>>,
    reader_rx: oneshot::Receiver<ReadHalf<TcpStream>>,

    writer_tx: Option<oneshot::Sender<WriteHalf<TcpStream>>>,
    writer_rx: oneshot::Receiver<WriteHalf<TcpStream>>,
}

impl SessionRetriever {
    pub fn new() -> Self {
        let (retrieve_tx, retrieve_rx) = broadcast::channel(1);
        let (egress_msg_tx, egress_msg_rx) = oneshot::channel();
        let (reader_tx, reader_rx) = oneshot::channel();
        let (writer_tx, writer_rx) = oneshot::channel();

        SessionRetriever {
            retrieve_tx,
            retrieve_rx,
            reader_tx: Some(reader_tx),
            reader_rx,
            egress_msg_tx: Some(egress_msg_tx),
            egress_msg_rx,
            writer_tx: Some(writer_tx),
            writer_rx,
        }
    }

    pub fn take(
        &mut self,
    ) -> (
        oneshot::Sender<mpsc::Sender<EgressMessage>>,
        oneshot::Sender<ReadHalf<TcpStream>>,
        oneshot::Sender<WriteHalf<TcpStream>>,
    ) {
        (
            self.egress_msg_tx.take().unwrap(),
            self.reader_tx.take().unwrap(),
            self.writer_tx.take().unwrap(),
        )
    }

    pub async fn retrieve(&self) {
        self.retrieve_tx.send(()).await;
    }
}

#[derive(Component)]
pub struct Session {
    egress_msg_tx: mpsc::Sender<EgressMessage>,
    egress_msg_rx: Option<mpsc::Receiver<EgressMessage>>,

    retriever: SessionRetriever,
}

impl Session {
    pub fn new() -> Self {
        let (egress_msg_tx, egress_msg_rx) = mpsc::channel(EGRESS_MESSAGE_BUFFER_SIZE);

        Session {
            egress_msg_tx,
            egress_msg_rx: Some(egress_msg_rx),
            retriever: SessionRetriever::new(),
        }
    }

    pub fn new_ctx(&self) -> SessionContext {
        SessionContext {
            egress_msg_tx: self.egress_msg_tx.clone(),
        }
    }

    pub fn start(
        &mut self,
        socket: TcpStream,
        ingress_msg_tx: mpsc::Sender<IngressMessage>,
        zone: &mut dyn AsyncContext<Zone>,
    ) {
        let (mut reader, mut writer) = tokio::io::split(socket);
        let mut egress_msg_rx = self
            .egress_msg_rx
            .take()
            .expect("out-message channel must be set before start");
        let (mut egress_msg_retriever, mut reader_retriever, mut writer_retriever)
            = self.retriever.take();

        // Start receiving-loop
        let mut retrieve_rx = self.retriever.retrieve_tx.subscribe();
        let ctx = self.new_ctx();

        zone.spawn(async move {
            loop {
                let mut header_buf = [0u8; HEADER_SIZE];
                let mut body_buf = Vec::new();

                if let Err(e) = Self::recv(&mut reader) {
                    eprintln!("Error receiving: {}", e);
                    //todo: Stop the session
                    break;
                }
            }
        });

        // Start sending-loop
        let mut retrieve_rx = self.retriever.retrieve_tx.subscribe();

        zone.spawn(async move {
            let mut egress_msg_buf = Vec::with_capacity(EGRESS_MESSAGE_BUFFER_SIZE);

            loop {
                if let Some(_) = retrieve_rx.try_recv() {
                    break;
                }

                egress_msg_rx
                    .recv_many(&mut egress_msg_buf, EGRESS_MESSAGE_BUFFER_SIZE)
                    .await;
                for msg in egress_msg_buf.drain(..) {
                    if let Err(e) = Session::send(&mut writer, msg).await {
                        eprintln!("Error sending: {}", e);
                        //todo: Stop the session
                        break;
                    }
                }
                
            }
            
            egress_msg_retriever.send(egress_msg_rx).await;
        });
    }

    pub async fn recv(
        reader: &mut ReadHalf<TcpStream>,
    ) -> Result<(ProtocolCategory, Bytes), std::io::Error> {
        let mut header_buf = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_buf).await?;
        let header = deserialize_header(&header_buf);

        let mut body_buf = BytesMut::with_capacity(header.length);
        reader.read_exact(&mut body_buf[..header.length]).await?;

        Ok((header.category, body_buf.freeze()))
    }

    pub async fn send(
        writer: &mut WriteHalf<TcpStream>,
        buffer: Bytes,
    ) -> Result<(), std::io::Error> {
        writer.write_all(&buffer[..]).await?;
        Ok(())
    }
}
