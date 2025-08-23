use bevy_ecs::prelude::*;

pub enum CombatCommand {
    None,
}

#[derive(Component)]
pub struct CombatController {
    command: CombatCommand,
}