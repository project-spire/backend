mod modify_resource;
mod modify_stat;

use std::time::{Duration, Instant};
use bevy_ecs::prelude::*;

use crate::calc::Modifier;
use crate::character::status::{Resource, Stat, State};

#[derive(Debug)]
pub struct EffectInstance {
    id: i64,
    creator: Option<Entity>,
    expire: Option<Instant>,

    trigger: EffectTrigger,
    trigger_count: u8,
    trigger_limit: Option<u8>,

    target: EffectTarget,

    effects: Vec<Effect>,
}

#[derive(Debug)]
pub enum Effect {
    ResourceModify {
        resource: Resource,
        modifier: Modifier<i64>,
    },
    StatModify {
        stat: Stat,
        modifier: Modifier<i64>,
    },
    StateAdd {
        state: State,
        count: u8,
    },
    StateErase {
        state: State,
    },
}

#[derive(Debug)]
pub enum EffectTrigger {
    Continuous,
    Periodic {
        period: Duration,
        next: Instant,
    },
    Conditional {
        condition: EffectCondition
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum EffectCondition {
    OnEffectAdded,
    OnEffectExpired,
    OnEntityKilled,
    OnEntityDying,
}

#[derive(Debug)]
pub enum EffectTarget {
    Source,
    Entities(Vec<Entity>),
    Region {
        exclude_source: bool,
        // shape:
        entities_max: Option<u8>,
    }
}

impl EffectInstance {
    pub fn new(
        creator: Option<Entity>,
        expire: Option<Instant>,
        trigger: EffectTrigger,
        trigger_limit: Option<u8>,
        target: EffectTarget,
        effects: Vec<Effect>,
    ) -> Self {
        Self {
            id: util::id::generate(),
            creator,
            expire,
            trigger,
            trigger_count: 0,
            trigger_limit,
            target,
            effects,
        }
    }

    pub fn can_apply(
        &self,
        condition: Option<&EffectCondition>,
        now: &Instant,
    ) -> bool {
        match &self.trigger {
            EffectTrigger::Continuous => {
                true
            }
            EffectTrigger::Periodic { period: _period, next } => {
                *now >= *next
            }
            EffectTrigger::Conditional { condition: my_condition } => {
                if let Some(condition) = condition {
                    condition == my_condition
                } else {
                    false
                }
            }
        }
    }

    pub fn apply(
        &mut self,
        world: &mut World,
        source: &Entity,
    ) {
        let targets = match &self.target {
            EffectTarget::Source => std::slice::from_ref(source),
            EffectTarget::Entities(entities) => &entities,
            EffectTarget::Region { .. } => unimplemented!(),
        };

        for target in targets {
            for effect in &self.effects {
                match effect {
                    Effect::ResourceModify { .. } => {}
                    Effect::StatModify { stat, modifier } => {
                        modify_stat::apply(world, target, &stat, modifier);
                    }
                    _ => unimplemented!()
                }
            }
        }
    }
    
    pub fn can_expire(&self, now: &Instant) -> bool {
        if let Some(expire) = &self.expire {
            now >= expire
        } else {
            false
        }
    }
    
    pub fn expire(
        &mut self,
        world: &mut World,
        source: &Entity,
    ) {
        if let EffectTrigger::Conditional { condition } = &self.trigger {
            if *condition == EffectCondition::OnEffectExpired {
                self.apply(world, source);
            }
        }
    }

    pub fn id(&self) -> i64 { self.id }
}
