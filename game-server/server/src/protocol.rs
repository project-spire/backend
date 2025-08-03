include!(concat!(env!("OUT_DIR"), "/spire.protocol.rs"));

pub mod auth {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.auth.rs"));

    pub fn encode_auth<T: prost::Message>(protocol: &T) -> Result<bytes::Bytes, crate::protocol::Error> {
        crate::protocol::encode::<T>(crate::protocol::Category::Auth, protocol)
    }
}

pub mod net {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.net.rs"));

    pub fn encode_net<T: prost::Message>(protocol: &T) -> Result<bytes::Bytes, crate::protocol::Error> {
        crate::protocol::encode::<T>(crate::protocol::Category::Net, protocol)
    }
}

pub mod game {
    include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.rs"));

    pub fn encode_game<T: prost::Message>(protocol: &T) -> Result<bytes::Bytes, crate::protocol::Error> {
        crate::protocol::encode::<T>(crate::protocol::Category::Game, protocol)
    }
}

pub mod category;
pub mod handler;

pub use category::Category;
pub use prost::Message;

use bytes::{BufMut, Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub const HEADER_SIZE: usize = 4;

pub struct Header {
    pub category: Category,
    pub length: usize,
}

impl Header {
    pub fn encode(buf: &mut BytesMut, category: Category, length: usize) -> Result<(), Error> {
        if buf.remaining_mut() < HEADER_SIZE {
            return Err(Error::NotEnoughBuffer(buf.remaining_mut(), HEADER_SIZE));
        }

        buf.put_u8(category as u8);
        buf.put_u8(0); // Reserved
        buf.put_u8((length >> 8) as u8);
        buf.put_u8(length as u8);

        Ok(())
    }

    pub fn decode(buf: &[u8; HEADER_SIZE]) -> Result<Self, Error> {
        let category = Category::decode(buf[0])?;
        let length =  ((buf[2] as usize) << 8) | (buf[3] as usize);

        Ok(Self { category, length })
    }
}

pub fn encode<T: prost::Message>(category: Category, protocol: &T) -> Result<Bytes, Error> {
    let length = protocol.encoded_len();
    if length > u16::MAX as usize {
        return Err(Error::ProtocolLength(length));
    }

    let mut buf = BytesMut::with_capacity(HEADER_SIZE + length);

    Header::encode(&mut buf, category, length)?;
    protocol.encode(&mut buf)?;

    Ok(buf.freeze())
}

#[derive(Debug)]
pub enum Error {
    InvalidCategory(u8),
    ProtocolLength(usize),
    NotEnoughBuffer(usize, usize),
    Encode(prost::EncodeError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCategory(c) => write!(f, "Invalid protocol category: {}", c),
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
