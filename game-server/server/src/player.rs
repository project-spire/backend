pub mod account;

use bevy_ecs::prelude::*;
use crate::database::{DatabaseClient, DatabaseError};
use crate::network::authenticator::Entry;
use self::account::Account;
use crate::character::*;
// use crate::character::movement::MovementController;
// use crate::character::stat::*;
// use crate::character::status_effect::*;
// use crate::physics::object::Transform;
// use crate::player::account::*;
// use std::error::Error;
// use tokio_postgres::Client;
//

#[derive(Bundle)]
pub struct PlayerData {
    // network
    pub account: Account,

    // character
    // pub character: Character,
    // pub character_stat: CharacterStat,
    // pub mobility_stat: MobilityStat,

    // movement
    // pub transform: Transform,
    // pub movement_controller: MovementController,
}

impl PlayerData {
    pub async fn load(
        client: &DatabaseClient,
        entry: &Entry,
    ) -> Result<Self, DatabaseError> {
        let character = Character::load(entry.character_id, client).await?;
        // let character_stat = CharacterStat::load(entry.character_id, client).await?;

        Ok(PlayerData {
            account,

            // character,
            // character_stat,
            //
            // transform: Transform::default(),
            // movement_controller: MovementController::default(),
        })
    }
}
