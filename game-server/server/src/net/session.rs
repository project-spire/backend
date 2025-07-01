use actix::AsyncContext;
use bevy_ecs::component::Component;
use bytes::{Bytes, BytesMut};
use crate::world::zone::Zone;
use protocol::{HEADER_SIZE, ProtocolCategory, deserialize_header};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

pub type InMessage = (SessionContext, ProtocolCategory, Bytes);
pub type OutMessage = Bytes;

#[derive(Clone)]
pub struct SessionContext {
    out_message_tx: mpsc::Sender<OutMessage>,
    is_stopped: Arc<AtomicBool>,
}

impl SessionContext {
    pub fn start(&mut self) {
        self.is_stopped.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn stop(&mut self) {
        self.is_stopped.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn send(&mut self, msg: OutMessage) {
        _ = self.out_message_tx.send(msg).await;
    }
}

#[derive(Component)]
pub struct Session {
    reader: Arc<Mutex<Option<ReadHalf<TcpStream>>>>,
    writer: Arc<Mutex<Option<WriteHalf<TcpStream>>>>,

    out_message_tx: mpsc::Sender<OutMessage>,
    out_message_rx: Arc<Mutex<Option<mpsc::Receiver<OutMessage>>>>,

    is_stopped: Arc<AtomicBool>,
}

impl Session {
    pub fn new() -> Self {
        let (out_message_tx, out_message_rx) = mpsc::channel::<OutMessage>(todo!("const 가져오기"));
        
        Session {
            reader: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            out_message_tx,
            out_message_rx: Arc::new(Mutex::new(Some(out_message_rx))),
            is_stopped: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start(
        &mut self,
        socket: TcpStream,
        in_message_tx: mpsc::Sender<InMessage>,
        zone: &mut dyn AsyncContext<Zone>
    ) {
        // if !self.is_stopped.compare_exchange(true, false, std::sync::atomic::Ordering::Relaxed) {
        //     return;
        // }
        let is_stopped = self.is_stopped.clone();

        let (mut reader, mut writer) = tokio::io::split(socket);
        let reader_return = self.reader.clone();
        let writer_return = self.writer.clone();
        
        let out_message_rx_return = self.out_message_rx.clone();

        let ctx = SessionContext {
            out_message_tx: self.out_message_tx.clone(),
            is_stopped: is_stopped.clone(),
        };

        // Start receiving-loop
        zone.spawn(async move {
            loop {
                if is_stopped.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                if let Err(e) = Self::recv(&mut reader) {
                    eprintln!("Error receiving: {}", e);
                    break;
                }
            }

            *reader_return.lock().await = Some(reader);
        });

        // Start sending-loop
        zone.spawn(async move {
            let out_message_rx = (*out_message_rx_return.lock().await)
                .take()
                .expect("OutMessage channel must be set before start");
            
            loop {
                if is_stopped.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                //todo
            }

            *writer_return.lock().await = Some(writer);
            *out_message_rx_return.lock().await = Some(out_message_rx);
        });
    }

    pub fn stop(&mut self) {
        self.is_stopped.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn recv(reader: &mut ReadHalf<TcpStream>) -> Result<(ProtocolCategory, Bytes), std::io::Error> {
        let mut header_buf = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_buf).await?;
        let header = deserialize_header(&header_buf);

        let mut body_buf = BytesMut::with_capacity(header.length);
        reader.read_exact(&mut body_buf[..header.length]).await?;

        Ok((header.category, body_buf.freeze()))
    }

    pub async fn send(writer: &mut WriteHalf<TcpStream>, buffer: Bytes) -> Result<(), std::io::Error> {
        writer.write_all(&buffer[..]).await?;
        Ok(())
    }
}
