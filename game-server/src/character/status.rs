use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy_ecs::prelude::*;
use bitflags::bitflags;

use crate::calc::*;
use crate::character::effect::EffectInstance;

#[derive(Component)]
pub struct Status {
    pub level: u16,
    pub exp: u64,
    pub karma: i64,

    pub state: State,
    
    pub resources: HashMap<Resource, RangedValue<u64>>,
    pub stats: HashMap<Stat, BasedValue<u64>>,
    
    pub effects: HashMap<i64, (StatusEffect, Option<Instant>)>,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct State: u64 {
        // Life
        const Dead     = 0x0000_0000_0000_0001;
        const Dying    = 0x0000_0000_0000_0002;

        // Control
        const Stunned  = 0x0000_0000_0000_0010;
        const Silenced = 0x0000_0000_0000_0020;
        const Bound    = 0x0000_0000_0000_0040;
        const Sleeping = 0x0000_0000_0000_0080;

        // Elements
        const Frozen   = 0x0000_0000_0001_0000;
        const Burning  = 0x0000_0000_0002_0000;

        // Cognition
        const Blinded  = 0x0000_0001_0000_0000;
        const Deafened = 0x0000_0002_0000_0000;
    }
}

#[derive(Debug)]
pub enum Resource {
    Health,
    Stamina,
    Mana,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Stat {
    // Resource
    HealthMax,
    HealthRegen,
    
    StaminaMax,
    StaminaRegen,
    
    ManaMax,
    ManaRegen,

    // Movement
    MovementSpeed,

    // Combat
    AccuracyRate,
    EvasionRate,

    AttackSpeedRate,
    
    Armor,

    DamageTakenIncreaseRate,
    DamageTakenDecreaseRate,
    DamageGivenIncreaseRate,
    DamageGivenDecreaseRate,

    // Crafting
}

#[derive(Debug)]
pub struct StatusEffect {
    instance: EffectInstance,
    form: StatusEffectForm,
}

#[derive(Debug)]
pub enum StatusEffectForm {
    Continuous,
    Periodic {
        period: Duration,
        next: Instant,
    },
    Conditional {
        condition: StatusEffectCondition
    },
}

#[derive(Debug)]
pub enum StatusEffectCondition {
    OnKilled,
    OnDying,
    OnExpired,
}

impl Status {
    pub fn add_effect(&mut self, effect: StatusEffect, lifetime: Option<Instant>) {
        match effect.form {
            StatusEffectForm::Continuous => {
                todo!();
            }
            StatusEffectForm::Periodic { period, next } => {
                todo!();
            }
            _ => {}
        }

        self.effects.insert(effect.instance.id(), (effect, lifetime));
    }

    pub fn erase_effect(&mut self, id: i64) {
        self.effects.remove(&id);
    }

    pub fn has_effect(&self, id: i64) -> bool {
        self.effects.get(&id).is_some()
    }

    pub fn recalculate(&mut self) {
        for stat in self.stats.values_mut() {
            stat.reset();
        }
    }
}

pub fn update(world: &mut World, mut query: Query<(Entity, &mut Status)>) {
    let now = Instant::now();

    query.iter_mut().for_each(|(entity, mut status)| {
        update_effects(world, &entity, &mut status, &now);
    })
}

fn update_effects(
    world: &mut World,
    entity: &Entity,
    status: &mut Status,
    now: &Instant
) {
    let mut is_dirty = false;

    status.effects.retain(|_, (effect, expiration)| {
        // Remove expired effects.
        if let Some(expiration) = expiration && *expiration >= *now {
            // Apply expiration conditional effects.
            if let StatusEffectForm::Conditional {condition} = &effect.form {
                if let StatusEffectCondition::OnExpired = condition {
                    effect.instance.apply(world, entity.clone(), None);
                    is_dirty = true;
                }
            }

            return false;
        }

        // Apply periodic effects.
        let (period, next) = match &mut effect.form {
            StatusEffectForm::Periodic { period, next } => (period, next),
            _ => return true,
        };

        if now < next {
            return true;
        }

        effect.instance.apply(world, entity.clone(), None);
        // next = next.checked_add(*period);

        is_dirty = true;
        true
    });

    if is_dirty {

    }
}
