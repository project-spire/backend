mod item;

use crate::config;
use crate::handler::ProtocolLocalHandler;
use crate::world::zone::Zone;
use bevy_ecs::entity::Entity;
use protocol::game::tool::{Cheat, CheatResult};
use protocol::game::tool::cheat::Kind;

impl ProtocolLocalHandler for Cheat {
    fn handle(self, entity: Entity, zone: &mut Zone) {
        if !config!(app).cheat.enabled {
            return;
        }

        let handle_result = match self.kind() {
            Kind::Item => item::handle(entity, zone, &self.arguments),
        };

        if let Some((result, message)) = handle_result {
            zone.send(entity, &CheatResult { result: result.into(), message });
        }
    }
}
