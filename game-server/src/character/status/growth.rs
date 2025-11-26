use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Growth {
    pub level: u16,
    pub exp: u64,
    pub karma: i64,
}
