use crate::handler::ProtocolLocalHandler;
use crate::net::session::SessionContext;
use bevy_ecs::prelude::*;
use protocol::game::social::PartyInvite;

impl ProtocolLocalHandler for PartyInvite {
    fn handle(self, world: &mut World, entity: Entity, ctx: SessionContext) {
        
    }
}