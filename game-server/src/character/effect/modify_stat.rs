use bevy_ecs::prelude::{Entity, World};

use crate::calc::Modifier;
use crate::character::status::{Stat, Status};

pub fn apply(
    world: &mut World,
    target: &Entity,
    stat: &Stat,
    modifier: &Modifier<i64>,
) {
    let mut status = match world.get_mut::<Status>(*target) {
        Some(status) => status,
        None => return,
    };

    let mut stat_value = match status.stats.get_mut(stat) {
        Some(value) => value,
        None => return,
    };

    stat_value.modfiy(modifier);
}