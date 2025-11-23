use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::World;

use crate::character::status::{State, Status};

pub fn apply(
    world: &mut World,
    target: &Entity,
    state: State,
) {
    let mut status = match world.get_mut::<Status>(*target) {
        Some(status) => status,
        None => return,
    };

    status.clear_state(state);
}
