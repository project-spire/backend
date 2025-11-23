mod resource_modify;
mod stat_modify;
mod state_add;
mod state_clear;

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
        priority: u8,
    },
    StateAdd {
        state: State,
        count: u8,
    },
    StateClear {
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
        now: &Instant,
    ) -> bool {
        match &self.trigger {
            EffectTrigger::Continuous => {
                true
            }
            EffectTrigger::Periodic { period: _period, next } => {
                *now >= *next
            }
            EffectTrigger::Conditional { .. } => {
                false
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
                    Effect::ResourceModify { resource, modifier } => {
                        resource_modify::apply(world, target, resource, modifier);
                    }
                    Effect::StatModify { stat, modifier, priority } => {
                        stat_modify::apply(world, target, stat, modifier, self.id, *priority);
                    }
                    Effect::StateAdd { state, count } => {
                        state_add::apply(world, target, *state, *count);
                    }
                    Effect::StateClear { state } => {
                        state_clear::apply(world, target, *state);
                    }
                }
            }
        }
    }

    pub fn can_expire(&self, now: &Instant) -> bool {
        if let Some(expire) = &self.expire && now >= expire {
            return true;
        }

        if let Some(trigger_limit) = &self.trigger_limit && self.trigger_count >= *trigger_limit {
            return true;
        }

        false
    }

    pub fn expire(
        &mut self,
        world: &mut World,
        source: &Entity,
    ) {
        self.trigger(world, source, &EffectCondition::OnEffectExpired);
    }

    pub fn trigger(
        &mut self,
        world: &mut World,
        source: &Entity,
        condition: &EffectCondition,
    ) {
        match &self.trigger {
            EffectTrigger::Conditional { condition: my_condition } => {
                if condition != my_condition {
                    return;
                }
            },
            _ => return,
        };

        if let Some(trigger_limit) = &self.trigger_limit && self.trigger_count >= *trigger_limit {
            return;
        }

        self.apply(world, source);
        self.trigger_count = self.trigger_count.saturating_add(1);
    }

    pub fn id(&self) -> i64 { self.id }
}
