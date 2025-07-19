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
        static STATEMENT: LazyLock<OnceCell<Statement>> = LazyLock::new(|| {
            OnceCell::new()
        });
        let statement = STATEMENT.get_or_try_init(|| async {
            client.prepare(
                "SELECT name, race \
                FROM character \
                WHERE id=$1 and account_id=$2"
            ).await
        }).await?;

        let row = client.query_one(statement, &[&character_id, &account_id]).await?;

        Ok(Character {
            id: character_id,
            name: row.get::<_, String>(0),
            race: row.get::<_, Race>(1),
        })
    }
}
