use crate::net::session::Entry;
use crate::world::zone::Zone;
use bevy_ecs::entity::Entity;

include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.handle.rs"));

mod net;
mod play;
mod tool;

pub trait ProtocolLocalHandler {
    fn handle(self, entity: Entity, zone: &mut Zone);
}

pub trait ProtocolGlobalHandler {
    fn handle(self, entry: Entry);
}
