use crate::character::movement::Movement;
use crate::handler::ProtocolLocalHandler;
use bevy_ecs::prelude::*;
use protocol::game::play::MovementCommand;

impl ProtocolLocalHandler for MovementCommand {
    fn handle(self, world: &mut World, entity: Entity) {
        if let Some(mut movement) = world.get_mut::<Movement>(entity) {
            movement.commands.push(self);
        };
    }
}
