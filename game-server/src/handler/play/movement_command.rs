use crate::character::status::movement::MovementCommands;
use crate::handler::ProtocolLocalHandler;
use crate::net::session::SessionContext;
use bevy_ecs::prelude::*;
use protocol::game::play::MovementCommand;

impl ProtocolLocalHandler for MovementCommand {
    fn handle(self, world: &mut World, entity: Entity, _: SessionContext) {
        if let Some(mut movement_commands) = world.get_mut::<MovementCommands>(entity) {
            movement_commands.queue.push(self);
        };
    }
}
