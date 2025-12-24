use super::Zone;
use crate::net::session::Session;
use crate::player::PlayerData;
use actix::prelude::*;
use bevy_ecs::prelude::*;
use protocol::game::net::ZoneTransfer;
use tracing::info;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct PlayerTransfer {
    pub session: Session,
    pub player_data: PlayerData,
}

#[derive(Component)]
pub struct PlayerTransferProcess {
    pub player_data: PlayerData,
}

impl Handler<PlayerTransfer> for Zone {
    type Result = ();

    fn handle(&mut self, msg: PlayerTransfer, ctx: &mut Self::Context) -> Self::Result {
        let PlayerTransfer { session, player_data } = msg;

        info!("{}: [{}] New player transfer started", self, session);

        session.send(&ZoneTransfer {
            zone_id: self.id,
        });

        let process = PlayerTransferProcess {
            player_data,
        };
        self.world.spawn((
            session,
            process,
        ));
    }
}
