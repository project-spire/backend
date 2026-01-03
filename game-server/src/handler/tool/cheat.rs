mod item;

use crate::config;
use crate::handler::ProtocolLocalHandler;
use crate::net::session::Session;
use bevy_ecs::prelude::*;
use protocol::game::tool::{Cheat, CheatResult};
use protocol::game::tool::cheat::Kind;

impl ProtocolLocalHandler for Cheat {
    fn handle(self, world: &mut World, entity: Entity, session: Session) {
        if !config!(app).cheat.enabled {
            return;
        }

        let handle_result = match self.kind() {
            Kind::Item => item::handle(world, entity, &self.arguments),
        };

        if let Some((result, message)) = handle_result {
            session.send(&CheatResult { result: result.into(), message });
        }
    }
}
