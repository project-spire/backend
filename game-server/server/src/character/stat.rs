use bevy_ecs::prelude::*;
use tokio_postgres::{Client, error::Error};

#[derive(Component)]
pub struct CharacterStat {
    // Level
    level: u16,
    exp: u32,

    // Core
    strength: u16,
    dexterity: u16,
    constitution: u16,
    intelligence: u16,

    // Optional
    faith: Option<u16>,
}

impl CharacterStat {
    pub async fn load(character_id: u64, client: &Client) -> Result<CharacterStat, Error> {
        let row = client.query_one(
            "SELECT level, exp, strength, dexterity, constitution, intelligence, faith \
            FROM character_stats WHERE character_id=$1",
            &[&(character_id as i64)],
        ).await?;

        Ok(CharacterStat {
            level: row.get::<_, i16>(0) as u16,
            exp: row.get::<_, i32>(1) as u32,
            strength: row.get::<_, i16>(2) as u16,
            dexterity: row.get::<_, i16>(3) as u16,
            constitution: row.get::<_, i16>(4) as u16,
            intelligence: row.get::<_, i16>(5) as u16,
            faith: row.get::<_, Option<i16>>(6).map(|v| v as u16)
        })
    }
}

#[derive(Component)]
pub struct MobilityStat {
    pub speed: f32,
    base_speed: f32,
}

#[derive(Component)]
pub struct CombatStat {
    pub attack: u32,
    base_attack: u32,
}

#[derive(Component)]
pub struct CraftingStat {

}