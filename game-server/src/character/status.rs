use std::collections::{LinkedList, HashMap};
use std::time::{Duration, Instant};

use bevy_ecs::prelude::*;
use bitflags::bitflags;

use crate::calc::*;

#[derive(Component)]
pub struct Status {
    level: u16,
    exp: u64,
    karma: i64,

    state: State,
    resources: HashMap<Resource, RangedValue<u32>>,

    attributes: HashMap<Attribute, BasedValue<u32>>,
    stats: HashMap<Stat, u32>,
    effects: LinkedList<(StatusEffect, Option<Instant>)>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct State: u64 {
        const Alive    = 0x0000_0000_0000_0001;
        const Stunned  = 0x0000_0000_0000_0002;
        const Silenced = 0x0000_0000_0000_0004;
    }
}

#[derive(Debug)]
pub enum Resource {
    Health,
    Mana,
    Rage,
}

#[derive(Debug)]
pub enum Attribute {
    // Core
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,

    // Extra
    Aura,
    Faith,
    Corruption,
    Spirit,
}

#[derive(Debug)]
pub enum Stat {
    // Movement
    MovementSpeed,
    StaminaMax,
    StaminaRegen,

    // Resource
    HealthRegen,
    ManaRegen,

    // Combat
    AccuracyRate,
    EvasionRate,

    AttackSpeedRate,

    TakenDamageIncreaseRate,
    TakenDamageDecreaseRate,
    GivenDamageIncreaseRate,
    GivenDamageDecreaseRate,

    // Crafting
}

#[derive(Debug)]
pub struct StatusEffect {
    effect: ,
    form: StatusEffectForm,
}

#[derive(Debug)]
pub enum StatusEffectForm {
    Static,
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
    OnDying,
    OnExpired,
}

impl Status {
    pub fn is_alive(&self) -> bool {
        self.state.contains(State::Alive)
    }

    pub fn add_effect(&mut self, effect: StatusEffect, lifetime: Option<Instant>) {
        match effect.form {
            StatusEffectForm::Static => {
                todo!();
            }
            StatusEffectForm::Periodic { period, next } => {
                todo!();
            }
            _ => {}
        }

        self.effects.push_back((effect, lifetime));
    }

    pub fn recalculate(&mut self) {

    }
}

impl Default for Status {
    fn default() -> Self {
        Status {
            level: u16::default(),
            exp: u64::default(),
            karma: i64::default(),

            state: State::Alive,
            resources: HashMap::default(),

            attributes: HashMap::default(),
            stats: HashMap::default(),

            effects: LinkedList::default(),
        }
    }
}

pub fn update(mut query: Query<&mut Status>) {
    let now = Instant::now();

    query.iter_mut().for_each(|(mut status)| {
        update_effects(&mut status, &now);
    })
}

fn update_effects(status: &mut Status, now: &Instant) {
    let mut cursor = status.effects.cursor_front_mut();

    while let Some((effect, expiration)) = cursor.current() {
        // Remove expired effects.
        if let Some(expiration) = expiration && *expiration >= *now {
            let (effect, _) = cursor.remove_current().unwrap();

            // Apply expiration conditional effects.
            if let StatusEffectForm::Conditional {condition} = effect.form {
                if let StatusEffectCondition::OnExpired = condition {
                    todo!();
                }
            }

            continue;
        }

        cursor.move_next();
    }
}
