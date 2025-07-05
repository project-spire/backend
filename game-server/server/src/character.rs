// pub mod audition;
// pub mod cognition;
// pub mod combat;
// pub mod movement;
// pub mod resource;
// pub mod stat;
// pub mod status_effect;
// pub mod vision;

use std::sync::LazyLock;
use bevy_ecs::prelude::*;
use crate::database::{DatabaseClient, DatabaseError, Statement};
use postgres_types::{FromSql, ToSql};
use tokio::sync::OnceCell;

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "race")]
pub enum Race {
    Human,
    Barbarian,
    Elf
}

#[derive(Debug, Component)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub race: Race,
}

impl Character {
    pub async fn load(character_id: u64, client: &DatabaseClient) -> Result<Character, DatabaseError> {
        static STATEMENT: LazyLock<OnceCell<Statement>> = LazyLock::new(|| {
            OnceCell::new()
        });
        let statement = STATEMENT.get_or_try_init(|| async {
            client.prepare(
                "SELECT name, race \
                FROM characters WHERE id=$1"
            ).await
        }).await?;

        let row = client.query_one(statement, &[&(character_id as i64)]).await?;

        Ok(Character {
            id: character_id,
            name: row.get::<_, String>(0),
            race: row.get::<_, Race>(1),
        })
    }
}
