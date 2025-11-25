use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Mana {
    pub current: u32,
    pub max: u32,
}