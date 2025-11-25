// pub mod audition;
// pub mod cognition;
// pub mod combat;
pub mod effect;
pub mod movement;
pub mod resource;
pub mod stats;
// pub mod vision;

use bevy_ecs::prelude::*;

use crate::db;
use data::character::Race;

#[derive(Debug, Component, sqlx::FromRow)]
pub struct Character {
    pub id: i64,
    pub name: Option<String>,
    pub race: Option<Race>,
    // pub location: Option<(u16, i64)>,
}

impl Character {
    pub async fn load(
        tx: &mut db::Transaction<'_>,
        character_id: &i64,
    ) -> Result<Character, db::Error> {
        let character = sqlx::query_as!(
            Character,
            r#"select id, name, race as "race: _"
            from character
            where id=$1"#,
            character_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(character)
    }
}
