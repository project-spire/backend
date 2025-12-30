use crate::character::Characters;
use crate::handler::ProtocolLocalHandler;
use crate::net::session::{Session, SessionContext};
use crate::net::zone::player_transfer::PlayerTransferProcess;
use bevy_ecs::prelude::*;
use protocol::game::net::ZoneTransferReady;

impl ProtocolLocalHandler for ZoneTransferReady {
    fn handle(self, world: &mut World, entity: Entity, ctx: SessionContext) {
        let character_id = {
            let Ok(mut entity) = world.get_entity_mut(entity) else {
                return;
            };

            let Some(process) = entity.take::<PlayerTransferProcess>() else {
                return;
            };

            entity.insert(process.player_data);

            match entity.get::<Session>() {
                Some(session) => session.character_id(),
                None => return,
            }
        };

        world.resource_mut::<Characters>().map.insert(character_id, entity);
    }
}