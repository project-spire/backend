use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Stamina {
    pub current: u32,
    pub max: u32,
}
