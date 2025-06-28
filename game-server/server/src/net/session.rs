use bytes::{Bytes, BytesMut};
use crate::protocol::{HEADER_SIZE, ProtocolCategory, deserialize_header};
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

pub type InMessage = (SessionPort, ProtocolCategory, Bytes);
pub type OutMessage = Bytes;

#[derive(Clone)]
pub struct SessionPort {
    pub out_message_tx: mpsc::Sender<OutMessage>,
    pub close_tx: mpsc::Sender<()>,
}

impl SessionPort {
    pub fn new(
        out_message_tx: mpsc::Sender<OutMessage>,
        close_tx: mpsc::Sender<()>,
    ) -> SessionPort {
        SessionPort {
            out_message_tx,
            close_tx,
        }
    }

    pub async fn close(&self) {
        _ = self.close_tx.send(()).await;
    }

    pub fn is_closed(&self) -> bool {
        self.close_tx.is_closed()
    }
}

pub async fn run_session(
    stream: TcpStream,
    in_message_tx: mpsc::Sender<InMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let peer_addr = stream
        .peer_addr()
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    let (reader, writer) = tokio::io::split(stream);

    let (out_message_tx, out_message_rx) = mpsc::channel(32);
    let (close_tx, close_rx) = mpsc::channel(1);
    let (retrieve_tx, retrieve_rx) = broadcast::channel(1);
    let port = SessionPort::new(out_message_tx, close_tx);

    println!("Session({}) has started", peer_addr);

    let retrieve_rx_recv = retrieve_rx.resubscribe();
    let retrieve_rx_send = retrieve_rx.resubscribe();
    tokio::spawn(async move {
        let (recv_result, send_result) = tokio::join!(
            recv(reader, in_message_tx, retrieve_rx_recv, port),
            send(writer, out_message_rx, retrieve_rx_send),
        );

        // println!("Session({}) has ended", peer_addr);
    });
}

enum RecvResult {
    Retrieve(ReadHalf<TcpStream>, mpsc::Sender<InMessage>),
    EOF,
    Error(Box<dyn Error + Send + Sync>),
}

async fn recv(
    mut reader: ReadHalf<TcpStream>,
    mut in_message_tx: mpsc::Sender<InMessage>,
    mut retrieve_rx: broadcast::Receiver<()>,
    port: SessionPort,
) -> RecvResult {
    loop {
        let mut header_buf = [0u8; HEADER_SIZE];
        match reader.read_exact(&mut header_buf).await {
            Ok(n) if n == 0 => return RecvResult::EOF,
            Ok(_) => {},
            Err(e) => return RecvResult::Error(e.into()),
        }
        let header = deserialize_header(&header_buf);

        let mut body_buf = BytesMut::with_capacity(header.length);
        match reader.read_exact(&mut body_buf[..header.length]).await {
            Ok(n) if n == 0 => return RecvResult::EOF,
            Ok(_) => {},
            Err(e) => return RecvResult::Error(e.into()),
        }

        _ = in_message_tx.send((port.clone(), header.category, body_buf.freeze())).await;

        if let Ok(_) = retrieve_rx.try_recv() {
            return RecvResult::Retrieve(reader, in_message_tx);
        }
    }
}

enum SendResult {
    Retrieve(WriteHalf<TcpStream>, mpsc::Receiver<OutMessage>),
    Error(Box<dyn Error + Send + Sync>),
    Closed,
}

async fn send(
    mut writer: WriteHalf<TcpStream>,
    mut out_message_rx: mpsc::Receiver<OutMessage>,
    mut retrieve_rx: broadcast::Receiver<()>,
) -> SendResult {
    loop {
        tokio::select! {
            m = out_message_rx.recv() => match m {
                Some(data) => {
                    if let Err(e) = writer.write_all(&data[..]).await {
                        return SendResult::Error(e.into());
                    }
                }
                None => return SendResult::Closed,
            },
            r = retrieve_rx.recv() => return match r {
                Ok(_) => SendResult::Retrieve(writer, out_message_rx),
                Err(_) => SendResult::Closed,
            }
        }
    }
}
