use actix::prelude::*;
use tracing::info;

use super::Zone;
use crate::player::PlayerData;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub player_data: PlayerData,
}

impl Handler<NewPlayer> for Zone {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        let player_data = msg.player_data;

        // Spawn on the world
        let character_id = player_data.character.id;
        let entity = self.world.spawn(player_data).id();

        self.characters.insert(character_id, entity);
        info!("{}: New player added", self);
    }
}
