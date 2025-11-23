use bevy_ecs::prelude::{Entity, World};

use crate::calc::{Modifiable, Modifier, ModifierInstance};
use crate::character::status::{Resource, Status};

pub fn apply(
    world: &mut World,
    target: &Entity,
    resource: &Resource,
    modifier: &Modifier<i64>,
) {
    let mut status = match world.get_mut::<Status>(*target) {
        Some(status) => status,
        None => return,
    };

    let resource_value = match status.resources.get_mut(resource) {
        Some(value) => value,
        None => return,
    };

    resource_value.add_modifier(ModifierInstance::new(0, modifier.clone(), 0));
}