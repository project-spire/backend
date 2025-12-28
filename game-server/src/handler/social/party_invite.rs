use crate::handler::ProtocolLocalHandler;
use bevy_ecs::prelude::*;
use protocol::game::social::PartyInvite;

impl ProtocolLocalHandler for PartyInvite {
    fn handle(self, world: &mut World, entity: Entity) {
        
    }
}