use bevy_ecs::prelude::*;

use crate::character::*;
use crate::net::session::Session;
use crate::world::transform::Transform;
// use crate::character::movement::MovementController;
// use crate::character::stat::*;
// use crate::character::status_effect::*;
// use crate::physics::object::Transform;

#[derive(Bundle)]
pub struct PlayerData {
    pub session: Session,
    pub character: Character,
    // pub character_stat: CharacterStat,
    // pub mobility_stat: MobilityStat,

    // movement
    pub transform: Transform,
    // pub movement_controller: MovementController,
}

impl PlayerData {
    pub async fn load(session: Session) -> Result<Self, db::Error> {
        let mut conn = db::get().await?;
        
        let character = Character::load(&mut conn, session.character_id()).await?;
        // let character_stat = CharacterStat::load(entry.character_id, client).await?;

        tx.commit().await?;

        Ok(PlayerData {
            session,
            character,
            // character_stat,
            transform: Transform::default(),
            // movement_controller: MovementController::default(),
        })
    }
}
