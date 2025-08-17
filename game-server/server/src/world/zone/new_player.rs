use actix::{Actor, Handler};
use quinn::{Connection, RecvStream, SendStream};
use tracing::info;
use crate::net::session::{Entry, Session, SessionContext};
use crate::player::PlayerData;
use super::Zone;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    entry: Entry,
    connection: Connection,
    player_data: PlayerData,
}

impl NewPlayer {
    pub fn new(
        entry: Entry,
        connection: Connection,
        streams: (SendStream, RecvStream),
        player_data: PlayerData
    ) -> Self {
        NewPlayer {
            entry,
            connection,
            player_data,
        }
    }
}

impl Handler<NewPlayer> for Zone {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, _: &mut Self::Context) -> Self::Result {
        let (entry, socket, player_data) = (msg.entry, msg.connection, msg.player_data);

        // Create a session
        let session = Session::new(entry.clone(), socket, self.ingress_proto_tx.clone());
        let egress_proto_tx = session.egress_tx.clone();
        let transfer_tx = session.transfer_tx.clone();
        let session_ctx = SessionContext {
            entry,
            session: session.start(),
            egress_tx: egress_proto_tx,
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
