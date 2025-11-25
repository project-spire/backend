use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::calc::*;

#[derive(Component)]
pub struct Stats {
    pub level: u16,
    pub exp: u64,
    pub karma: i64,

    pub stats: HashMap<Stat, BasedValue<i64>>,
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
