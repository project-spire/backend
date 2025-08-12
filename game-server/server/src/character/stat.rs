use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Debug, Default, Clone, Copy)]
pub struct Chance(pub u16);

impl Chance {
    pub fn hit(&self) -> bool {
        rand::rng().random_range(0..=10000) >= self.0
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Multiplier(pub u32);

impl Multiplier {
    pub fn calc(&self, value: i64) -> i64 {
        (value * (self.0 + 10000) as i64) / 10000
    }
}

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
    pub critical_damage_multiplier: Multiplier,
}

#[derive(Component)]
pub struct Stat {
    level: u16,
    exp: u64,
    karma: i64,

    core_base: CoreStatBlock,
    core_final: CoreStatBlock,

    combat_base: CombatStatBlock,
    combat_final: CombatStatBlock,
}
