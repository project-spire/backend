// pub mod audition;
// pub mod cognition;
// pub mod combat;
pub mod movement;
// pub mod resource;
// pub mod stat;
// pub mod status_effect;
// pub mod vision;

use bevy_ecs::prelude::*;
use postgres_types::{FromSql, ToSql};
use crate::database::{DatabaseClient, DatabaseError};

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "race")]
pub enum Race {
    Human,
    Barbarian,
}

#[derive(Debug, Component)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub race: Race,
}

impl Character {
    pub async fn load(
        character_id: i64,
        account_id: i64,
        client: &DatabaseClient
    ) -> Result<Character, DatabaseError> {
        let row = client.query_one(
            "SELECT name, race \
            FROM character \
            WHERE id=$1 and account_id=$2",
            &[&character_id, &account_id]
        ).await?;

        Ok(Character {
            id: character_id,
            name: row.get::<_, String>(0),
            race: row.get::<_, Race>(1),
        })
    }
}
