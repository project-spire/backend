pub mod account;

// use bevy_ecs::prelude::*;
// use crate::character::*;
// use crate::character::movement::MovementController;
// use crate::character::stat::*;
// use crate::character::status_effect::*;
// use crate::physics::object::Transform;
// use crate::player::account::*;
// use std::error::Error;
// use tokio_postgres::Client;
//
// #[derive(Bundle)]
// pub struct PlayerBundle {
//     // network
//     pub account: Account,
//
//     // character
//     pub character: Character,
//     pub character_stat: CharacterStat,
//     // pub mobility_stat: MobilityStat,
//
//     // movement
//     pub transform: Transform,
//     pub movement_controller: MovementController,
// }

// impl PlayerBundle {
//     pub async fn load(
//         account: Account,
//         character_id: u64,
//         client: &Client,
//     ) -> Result<Box<Self>, Box<dyn Error>> {
//         let character = Character::load(character_id, client).await?;
//         let character_stat = CharacterStat::load(character_id, client).await?;
//
//         Ok(Box::new(PlayerBundle {
//             account,
//
//             character,
//             character_stat,
//
//             transform: Transform::default(),
//             movement_controller: MovementController::default(),
//         }))
//     }
// }
