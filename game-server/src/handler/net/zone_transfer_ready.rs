use crate::character::Characters;
use crate::handler::ProtocolLocalHandler;
use crate::net::session::{Session, SessionContext};
use crate::net::zone::player_transfer::PlayerTransferProcess;
use bevy_ecs::prelude::*;
use protocol::game::net::ZoneTransferReady;

impl ProtocolLocalHandler for ZoneTransferReady {
    fn handle(self, world: &mut World, entity: Entity, ctx: SessionContext) {
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            let Some(process) = entity.take::<PlayerTransferProcess>() else {
                return;
            };
            
            entity.insert(process.player_data);
        } else {
            return;
        };
        
        world.resource_mut::<Characters>().map.insert(ctx.entry.character_id, entity);
        
        todo!("Send result");
    }
}