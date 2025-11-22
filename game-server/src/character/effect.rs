mod modify_resource;
mod modify_stat;

use bevy_ecs::prelude::*;

use crate::calc::Modifier;
use crate::character::status::{Resource, Stat};

#[derive(Debug)]
pub struct EffectInstance {
    id: i64,
    creator: Option<Entity>,
    effect: Effect,
}

#[derive(Debug)]
pub enum Effect {
    ModifyResource {
        resource: Resource,
        modifier: Modifier<u64>,
    },
    ModifyStat {
        stat: Stat,
        modifier: Modifier<u64>,
    },
}

#[derive(Debug)]
pub enum EffectTarget {
    Single(Entity),
    Multi(Vec<Entity>),
    Region {
        exclude_source: bool,
        // shape: 
    }
}

impl EffectInstance {
    pub fn id(&self) -> i64 { self.id }
    
    pub fn apply(
        &self,
        world: &mut World,
        source: Entity,
        target: &EffectTarget,
    ) {
        match &self.effect {
            Effect::ModifyResource { .. } => {}
            Effect::ModifyStat { stat, modifier } => {
                modify_stat::apply(world, target, &stat, modifier);
            }
        }
    }
}
