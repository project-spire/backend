use bevy_ecs::prelude::{Entity, World};

use crate::calc::{Modifiable, Modifier, ModifierInstance};
use crate::character::status::{Stat, Status};

pub fn apply(
    world: &mut World,
    target: &Entity,
    stat: &Stat,
    modifier: &Modifier<i64>,
    effect_id: i64,
    priority: u8,
) {
    let mut status = match world.get_mut::<Status>(*target) {
        Some(status) => status,
        None => return,
    };
    
    if let Some(stat_value) = status.stats.get_mut(stat) {
        stat_value.add_modifier(ModifierInstance::new(effect_id, modifier.clone(), priority));
    }
}
