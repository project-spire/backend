use crate::net::session::Entry;
use crate::world::zone::Zone;

include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.handle.rs"));

pub mod net;
pub mod play;

pub trait ProtocolLocalHandler {
    fn handle(self, zone: &mut Zone);
}

pub trait ProtocolGlobalHandler {
    fn handle(self, entry: Entry);
}
