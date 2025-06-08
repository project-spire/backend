use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Health {
    value: u32,
    max_value: u32,
}

#[derive(Component)]
pub struct Mana {
    value: u32,
    max_value: u32,
}

#[derive(Component)]
pub struct Rage {
    value: u32,
    max_value: u32,
}
