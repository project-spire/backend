#![allow(static_mut_refs)]

mod error;
mod link;
mod parse;

pub mod prelude {
    pub use crate::{link::DataId, link::Link};
}

include!(concat!(env!("OUT_DIR"), "/spire.data.rs"));

pub mod character {
    include!(concat!(env!("OUT_DIR"), "/spire.data.character.rs"));
}

pub mod item {
    include!(concat!(env!("OUT_DIR"), "/spire.data.item.rs"));
}
