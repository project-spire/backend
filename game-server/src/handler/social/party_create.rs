use crate::handler::ProtocolLocalHandler;
use bevy_ecs::prelude::*;
use protocol::game::social::PartyCreate;

impl ProtocolLocalHandler for PartyCreate {
    fn handle(self, world: &mut World, entity: Entity) {

    }
}