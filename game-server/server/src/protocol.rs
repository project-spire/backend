include!(concat!(env!("OUT_DIR"), "/spire.protocol.rs"));
include!(concat!(env!("OUT_DIR"), "/spire.protocol.impl.rs"));

pub mod auth {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.auth.rs"));
}

pub mod net {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.net.rs"));
}

pub mod play {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.play.rs"));
}

pub mod handler;
pub mod convert;

use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
use std::fmt::{Display, Formatter};

pub const HEADER_SIZE: usize = 4;

pub struct Header {
    pub length: usize,
    pub id: u16,
}

impl Header {
    pub fn encode(
        buf: &mut BytesMut,
        length: usize,
        id: u16,
    ) -> Result<(), Error> {
        if buf.remaining_mut() < HEADER_SIZE {
            return Err(Error::NotEnoughBuffer(buf.remaining_mut(), HEADER_SIZE));
        }

        buf.put_u8((length >> 8) as u8);
        buf.put_u8(length as u8);
        buf.put_u8((id >> 8) as u8);
        buf.put_u8(id as u8);

        Ok(())
    }

    pub fn decode(buf: &[u8; HEADER_SIZE]) -> Result<Self, Error> {
        let length =  ((buf[0] as usize) << 8) | (buf[1] as usize);
        let protocol =  ((buf[2] as u16) << 8) | (buf[3] as u16);

        Ok(Self { length, id: protocol })
    }
}

pub trait HasProtocolId {
    fn protocol_id(&self) -> u16;
}

pub fn encode(protocol: &(impl prost::Message + HasProtocolId)) -> Result<Bytes, Error>
{
    let length = protocol.encoded_len();
    if length > u16::MAX as usize {
        return Err(Error::ProtocolLength(length));
    }

    let mut buf = BytesMut::with_capacity(HEADER_SIZE + length);

    Header::encode(
        &mut buf,
        length,
        protocol.protocol_id(),
    )?;
    protocol.encode(&mut buf)?;

    Ok(buf.freeze())
}

#[derive(Debug)]
pub enum Error {
    ProtocolLength(usize),
    ProtocolId(u16),
    NotEnoughBuffer(usize, usize),
    Encode(prost::EncodeError),
    Decode(prost::DecodeError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolLength(len) => write!(f, "Protocol length of {} is too long", len),
            Self::ProtocolId(id) => write!(f, "Invalid protocol id: {}", id),
            Self::NotEnoughBuffer(prepared, required) => write!(f, "Not enough buffer size {prepared} for {required}"),
            Self::Encode(e) => write!(f, "{e}"),
            Self::Decode(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<prost::EncodeError> for Error {
    fn from(value: prost::EncodeError) -> Self {
        Self::Encode(value)
    }
}

impl From<prost::DecodeError> for Error {
    fn from(value: prost::DecodeError) -> Self {
        Self::Decode(value)
    }
}
