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

pub use prost::Message;

use bytes::{BufMut, Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub const HEADER_SIZE: usize = 4;

pub trait Protocol : Sized + prost::Message {
    fn protocol() -> u16;
}

pub struct Header {
    pub length: usize,
    pub protocol: u16,
}

impl Header {
    pub fn encode(
        buf: &mut BytesMut,
        length: usize,
        protocol: u16,
    ) -> Result<(), Error> {
        if buf.remaining_mut() < HEADER_SIZE {
            return Err(Error::NotEnoughBuffer(buf.remaining_mut(), HEADER_SIZE));
        }

        buf.put_u8((length >> 8) as u8);
        buf.put_u8(length as u8);
        buf.put_u8((protocol >> 8) as u8);
        buf.put_u8(protocol as u8);

        Ok(())
    }

    pub fn decode(buf: &[u8; HEADER_SIZE]) -> Result<Self, Error> {
        let length =  ((buf[0] as usize) << 8) | (buf[1] as usize);
        let protocol =  ((buf[2] as u16) << 8) | (buf[3] as u16);

        Ok(Self { length, protocol })
    }
}

pub fn encode<P>(protocol: &P) -> Result<Bytes, Error>
where P: Protocol + prost::Message,
{
    let length = protocol.encoded_len();
    if length > u16::MAX as usize {
        return Err(Error::ProtocolLength(length));
    }

    let mut buf = BytesMut::with_capacity(HEADER_SIZE + length);

    Header::encode(
        &mut buf,
        length,
        P::protocol(),
    )?;
    protocol.encode(&mut buf)?;

    Ok(buf.freeze())
}

#[derive(Debug)]
pub enum Error {
    ProtocolLength(usize),
    NotEnoughBuffer(usize, usize),
    Encode(prost::EncodeError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolLength(len) => write!(f, "Protocol length of {} is too long", len),
            Self::NotEnoughBuffer(prepared, required) => write!(f, "Not enough buffer size {prepared} for {required}"),
            Self::Encode(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<prost::EncodeError> for Error {
    fn from(value: prost::EncodeError) -> Self {
        Self::Encode(value)
    }
}
