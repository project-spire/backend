mod item;

use crate::handler::ProtocolLocalHandler;
use crate::world::zone::Zone;
use bevy_ecs::entity::Entity;
use protocol::game::tool::{Cheat, CheatResult};
use protocol::game::tool::cheat::Kind;

impl ProtocolLocalHandler for Cheat {
    fn handle(self, entity: Entity, zone: &mut Zone) {
        let (result, message) = match self.kind() {
            Kind::Item => item::handle(entity, zone, &self.arguments),
        };

        zone.send(entity, &CheatResult { result: result.into(), message });
    }
}
