// pub mod audition;
// pub mod cognition;
// pub mod combat;
pub mod effect;
pub mod equipment;
pub mod inventory;
pub mod movement;
pub mod path_tree;
pub mod sense;
pub mod status;
pub mod skill_set;
// pub mod vision;

use bevy_ecs::prelude::*;
use data::character::Race;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use util::id::Id;

#[derive(Debug, Component, Queryable, Selectable)]
#[diesel(table_name = data::schema::character)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Character {
    pub id: Id,
    pub name: String,
    pub race: Race,
}

#[derive(Resource, Default)]
pub struct Characters {
    pub map: HashMap<Id, Entity>,
}

impl Character {
    pub async fn load(
        conn: &mut db::Connection,
        character_id: Id,
    ) -> Result<Character, db::Error> {
        use data::schema::character::dsl::*;

        let c = character
            .filter(id.eq(character_id))
            .select(Character::as_select())
            .first(conn)
            .await?;

        Ok(c)
    }
}
