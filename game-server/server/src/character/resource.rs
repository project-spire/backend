use std::collections::HashMap;
use bevy_ecs::prelude::*;

#[derive(Debug, Default)]
pub struct ResourceBlock {
    health: u64,
    mana: Option<u64>,
    rage: Option<u64>,
}

#[derive(Component)]
pub struct Resource {
    current: ResourceBlock,
    max: ResourceBlock,
}
