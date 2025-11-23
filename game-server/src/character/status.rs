use std::collections::HashMap;
use std::time::Instant;

use bevy_ecs::prelude::*;

use crate::calc::*;
use crate::character::effect::{EffectCondition, EffectInstance};

#[derive(Component)]
pub struct Status {
    pub level: u16,
    pub exp: u64,
    pub karma: i64,

    pub states: [u8; State::Size as usize],
    
    pub resources: HashMap<Resource, BasedRange<i64>>,
    pub stats: HashMap<Stat, BasedValue<i64>>,
    
    pub effects: HashMap<i64, EffectInstance>,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum State {
    // Life
    Dead = 0,
    Dying = 1,

    // Control
    Stunned = 2,
    Silenced = 3,
    Bound = 4,
    Sleeping = 5,

    // Elements
    Frozen = 6,
    Burning = 7,

    // Cognition
    Blinded = 8,
    Deafened = 9,

    Size = 10,
}

#[derive(Debug, Hash, PartialEq, Eq)]
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

impl Status {
    pub fn add_effect(
        &mut self,
        world: &mut World,
        entity: &Entity,
        mut effect: EffectInstance,
    ) {
        effect.trigger(world, entity, &EffectCondition::OnEffectAdded);

        self.effects.insert(effect.id(), effect);
    }

    pub fn trigger_effect(
        &mut self,
        world: &mut World,
        entity: &Entity,
        condition: &EffectCondition,
    ) {
        for effect in self.effects.values_mut() {
            effect.trigger(world, entity, condition);
        }
    }

    pub fn erase_effect(&mut self, id: i64) {
        self.effects.remove(&id);
    }

    pub fn has_effect(&self, id: i64) -> bool {
        self.effects.get(&id).is_some()
    }

    pub fn has_state(&self, state: State) -> bool {
        self.states[state as usize] > 0
    }

    pub fn add_state(&mut self, state: State, count: u8) {
        self.states[state as usize] = self.states[state as usize].saturating_add(count);
    }

    pub fn sub_state(&mut self, state: State, count: u8) {
        self.states[state as usize] = self.states[state as usize].saturating_sub(count);
    }

    pub fn clear_state(&mut self, state: State) {
        self.states[state as usize] = 0;
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
    source: &Entity,
    status: &mut Status,
    now: &Instant
) {
    status.effects.retain(|_, effect| {
        // Remove expired effects.
        if effect.can_expire(now) {
            effect.expire(world, source);
            return false;
        }

        if effect.can_apply(now) {
            effect.apply(world, source);
        }

        true
    });
}
