use bevy_ecs::prelude::*;
use crate::calc::RangedValue;

#[derive(Debug, Default)]
pub struct ResourceBlock {
    health: u64,
    mana: Option<u64>,
    rage: Option<u64>,
}

#[derive(Component)]
pub struct Health {
    pub value: RangedValue<i64>,
}
