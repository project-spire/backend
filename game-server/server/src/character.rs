// pub mod audition;
// pub mod cognition;
// pub mod combat;
pub mod movement;
// pub mod resource;
// pub mod stat;
// pub mod status_effect;
// pub mod vision;

use bevy_ecs::prelude::*;
use gel_derive::Queryable;
use serde::Deserialize;
use uuid::Uuid;
use crate::db::{DbClient, DbError};

#[derive(Debug, Queryable)]
pub enum Race {
    Human,
    Barbarian,
}

#[derive(Debug, Component, Queryable)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub race: Race,
}

impl Character {
    pub async fn load(
        client: &DbClient,
        character_id: &Uuid
    ) -> Result<Character, DbError> {
        let query = "
            SELECT Character {
                id,
                name,
                race
            }
            FILTER .id = <uuid>$0
            LIMIT 1;
        ";

        let character = client.query_required_single(query, &(character_id,)).await?;
        Ok(character)
    }
}
