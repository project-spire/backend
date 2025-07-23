use actix::{Actor, Handler};
use tokio::net::TcpStream;
use tracing::info;
use crate::network::session::{Entry, Session, SessionContext};
use crate::player::PlayerData;
use super::Zone;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    entry: Entry,
    socket: TcpStream,
    player_data: PlayerData,
}

impl NewPlayer {
    pub fn new(entry: Entry, socket: TcpStream, player_data: PlayerData) -> Self {
        NewPlayer {
            entry,
            socket,
            player_data,
        }
    }
}

impl Handler<NewPlayer> for Zone {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, _: &mut Self::Context) -> Self::Result {
        let (entry, socket, player_data) = (msg.entry, msg.socket, msg.player_data);

        // Create a session
        let session = Session::new(entry.clone(), socket, self.ingress_proto_tx.clone());
        let egress_proto_tx = session.egress_proto_tx.clone();
        let transfer_tx = session.transfer_tx.clone();
        let session_ctx = SessionContext {
            entry,
            session: session.start(),
            egress_proto_tx,
            transfer_tx,
        };

        // Spawn on the world
        let character_id = player_data.character.id;
        let entity = self.world.spawn(player_data).id();
        if let Ok(mut entity) = self.world.get_entity_mut(entity) {
            entity.insert(session_ctx);
        }

        self.characters.insert(character_id, entity);
        info!("{}: New player added", self);
    }
}
