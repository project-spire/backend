pub mod audition;
pub mod cognition;
pub mod combat;
pub mod movement;
pub mod resource;
pub mod stat;
pub mod status_effect;
pub mod vision;


use bevy_ecs::prelude::*;
use postgres_types::{FromSql, ToSql};
use tokio_postgres::{Client, Error};

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "race")]
pub enum Race {
    Human,
    Barbarian,
    Elf
}

#[derive(Component)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub race: Race,
}

impl Character {
    pub async fn load(character_id: u64, client: &Client) -> Result<Character, Error> {
        let row = client.query_one(
            "SELECT name, race \
            FROM characters WHERE id=$1",
            &[&(character_id as i64)],
        ).await?;

        Ok(Character {
            id: character_id,
            name: row.get::<_, String>(0),
            race: row.get::<_, Race>(1),
        })
    }
}
