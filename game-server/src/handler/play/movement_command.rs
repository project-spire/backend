use actix::prelude::*;
use tracing::error;

use crate::character::movement;
use crate::handler::ProtocolHandler;
use crate::net::session::SessionContext;
use crate::world::zone::{self, Zone};
use protocol::game::play::MovementCommand;
use util::timestamp::Timestamp;

impl ProtocolHandler for MovementCommand {
    fn handle(self, ctx: &SessionContext) {
        let zone: Addr<Zone> = todo!();
        zone.do_send(zone::MovementCommand::new(self, ctx.clone()));
    }
}
