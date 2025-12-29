include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.impl.rs"));

pub mod auth {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.auth.rs"));
}

pub mod net {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.net.rs"));
}

pub mod tool {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.tool.rs"));
}

pub mod play {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.play.rs"));
}

pub mod social {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.social.rs"));
}

use bytes::{BufMut, Bytes, BytesMut};

pub type ProtocolId = u16;

pub struct Header {
    pub length: u16,
    pub id: ProtocolId,
}

pub trait Protocol {
    fn protocol_id(&self) -> ProtocolId;
}

pub enum ProtocolHandler {
    Local,
    Global,
}

impl Header {
    pub const fn size() -> usize { 4 }

    pub fn encode(buffer: &mut BytesMut, length: usize, id: ProtocolId) -> Result<(), Error> {
        if buffer.remaining_mut() < Self::size() {
            return Err(Error::NotEnoughBuffer(buffer.remaining_mut(), Self::size()));
        }

        buffer.put_u8((length >> 8) as u8);
        buffer.put_u8(length as u8);
        buffer.put_u8((id >> 8) as u8);
        buffer.put_u8(id as u8);

        Ok(())
    }

    pub fn decode(buffer: &[u8]) -> Result<Self, Error> {
        let length = ((buffer[0] as u16) << 8) | (buffer[1] as u16);
        let id = ((buffer[2] as u16) << 8) | (buffer[3] as u16);

        Ok(Self {
            length,
            id,
        })
    }
}

pub fn encode(protocol: &(impl prost::Message + Protocol)) -> Result<Bytes, Error> {
    let length = protocol.encoded_len();
    if length > u16::MAX as usize {
        return Err(Error::ProtocolLength(length));
    }

    let mut buffer = BytesMut::with_capacity(Header::size() + length);

    Header::encode(&mut buffer, length, protocol.protocol_id())?;
    protocol.encode(&mut buffer)?;

    Ok(buffer.freeze())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid protocol length: {0}")]
    ProtocolLength(usize),

    #[error("Invalid protocol ID: {0}")]
    ProtocolId(ProtocolId),

    #[error("Not enough buffer size {0} for {1}")]
    NotEnoughBuffer(usize, usize),

    #[error("Failed to encode: {0}")]
    Encode(#[from] prost::EncodeError),

    #[error("Failed to decode: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("Unhandled protocol id: {0}")]
    UnhandledProtocol(ProtocolId),
}
