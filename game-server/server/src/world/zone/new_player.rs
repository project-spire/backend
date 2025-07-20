use actix::{Actor, Handler};
use tokio::net::TcpStream;
use tracing::info;
use crate::network::session::Session;
use crate::player::PlayerData;
use super::Zone;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub socket: TcpStream,
    pub player_data: PlayerData
}

impl Handler<NewPlayer> for Zone {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        let character_id = msg.player_data.character.id;
        let entity = self.world.spawn(msg.player_data).id();
        self.characters.insert(character_id, entity);
        
        Session::new(msg.socket, self.ingress_proto_tx.clone()).start();
        
        info!("{}: New player added", self);
    }
}
