pub mod account;

use bevy_ecs::prelude::*;
use crate::character::*;
use crate::db::{DbClient, DbError};
use crate::net::session::Entry;
use crate::world::transform::Transform;
use self::account::Account;
// use crate::character::movement::MovementController;
// use crate::character::stat::*;
// use crate::character::status_effect::*;
// use crate::physics::object::Transform;

#[derive(Bundle)]
pub struct PlayerData {
    pub account: Account,
    pub character: Character,
    // pub character_stat: CharacterStat,
    // pub mobility_stat: MobilityStat,

    // movement
    pub transform: Transform,
    // pub movement_controller: MovementController,
}

impl PlayerData {
    pub async fn load(
        client: &DbClient,
        entry: &Entry,
    ) -> Result<Self, DbError> {
        let account = Account { account_id: entry.account_id };
        let character = Character::load(client, &entry.character_id).await?;
        // let character_stat = CharacterStat::load(entry.character_id, client).await?;

        Ok(PlayerData {
            account,
            character,
            // character_stat,
            transform: Transform::default(),
            // movement_controller: MovementController::default(),
        })
    }
}
