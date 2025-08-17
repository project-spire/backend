use bevy_ecs::prelude::*;
use crate::calc::Chance;

#[derive(Debug)]
pub enum CoreStatType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Faith,
}

#[derive(Debug, Default)]
pub struct CoreStatBlock {
    pub strength: u16,
    pub dexterity: u16,
    pub constitution: u16,
    pub intelligence: u16,
    pub faith: Option<u16>,
    pub spirit: Option<u16>,
}

#[derive(Debug)]
pub enum CombatStatType {
    AttackSpeed,
    CastSpeed,
    CriticalChance,
    CriticalDamageMultiplier,
}

#[derive(Debug, Default)]
pub struct CombatStatBlock {
    pub attack_speed: u16,
    pub cast_speed: u16,
    pub critical_chance: Chance,
    pub critical_damage_multiplier: f32,
}

#[derive(Component, Default)]
pub struct Stat {
    level: u16,
    exp: u64,
    karma: i64,

    core_base: CoreStatBlock,
    core_final: CoreStatBlock,

    combat_base: CombatStatBlock,
    combat_final: CombatStatBlock,
}
