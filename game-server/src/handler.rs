use crate::net::session::Session;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.handle.rs"));

mod net;
mod play;
mod social;
mod tool;

pub trait ProtocolLocalHandler {
    fn handle(self, world: &mut World, entity: Entity, session: Session);
}

pub trait ProtocolGlobalHandler {
    fn handle(self, session: Session);
}
