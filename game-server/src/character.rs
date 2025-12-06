// pub mod audition;
// pub mod cognition;
// pub mod combat;
pub mod effect;
pub mod equipment;
pub mod inventory;
pub mod path_tree;
pub mod sense;
pub mod status;
pub mod skill_set;
// pub mod vision;

use bevy_ecs::prelude::*;
use data::character::Race;

#[derive(Debug, Component)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub race: Race,
}

impl Character {
    pub async fn load(
        conn: &mut db::Connection,
        character_id: i64,
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
