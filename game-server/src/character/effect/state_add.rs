use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::World;

use crate::character::status::{State, Status};

pub fn apply(
    world: &mut World,
    target: &Entity,
    state: State,
    count: u8,
) {
    let mut status = match world.get_mut::<Status>(*target) {
        Some(status) => status,
        None => return,
    };
    
    status.add_state(state, count);
}
