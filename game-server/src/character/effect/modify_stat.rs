use bevy_ecs::prelude::{Entity, World};

use crate::calc::Modifier;
use crate::character::effect::EffectTarget;
use crate::character::status::{Stat, Status};

pub fn apply(
    world: &mut World,
    target: &EffectTarget,
    stat: &Stat,
    modifier: &Modifier<u64>,
) {
    let mut process = |entity: &Entity| {
        let mut status = match world.get_mut::<Status>(*entity) {
            Some(status) => status,
            None => return,
        };

        let mut stat_value = match status.stats.get_mut(stat) {
            Some(value) => value,
            None => return,
        };

        stat_value.modfiy(modifier);
    };

    match target {
        EffectTarget::Single(entity) => {
            process(entity);
        }
        EffectTarget::Multi(entities) => {
            for entity in entities {
                process(entity);
            }
        }
        EffectTarget::Region { .. } => {
            todo!();
        }
    }
}