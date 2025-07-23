pub mod account;

use bevy_ecs::prelude::*;
use crate::character::*;
use crate::database::{DatabaseClient, DatabaseError};
use crate::network::session::Entry;
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
        client: &DatabaseClient,
        entry: &Entry,
    ) -> Result<Self, DatabaseError> {
        let account = Account { account_id: entry.account_id, privilege: entry.privilege };
        let character = Character::load(entry.character_id, entry.account_id, client).await?;
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
