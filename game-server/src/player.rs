use bevy_ecs::prelude::*;

use crate::character::*;
use crate::character::skill_tree::SkillTree;
use crate::net::session::{Entry, Session};
use crate::world::transform::Transform;
// use crate::character::movement::MovementController;
// use crate::character::stat::*;
// use crate::character::status_effect::*;
// use crate::physics::object::Transform;

#[derive(Bundle)]
pub struct PlayerData {
    pub character: Character,
    pub path_tree: SkillTree,
    // pub character_stat: CharacterStat,
    // pub mobility_stat: MobilityStat,

    pub transform: Transform,
    // pub movement_controller: MovementController,
}

impl PlayerData {
    pub async fn load(entry: &Entry) -> Result<Self, db::Error> {
        let mut conn = db::conn().await?;
        
        let character_id = entry.character_id;
        let character = Character::load(&mut conn, character_id).await?;
        let path_tree = SkillTree::load(&mut conn, character_id).await?;
        
        // let character_stat = CharacterStat::load(entry.character_id, client).await?;

        Ok(PlayerData {
            character,
            path_tree,
            // character_stat,
            transform: Transform::default(),
            // movement_controller: MovementController::default(),
        })
    }
}
