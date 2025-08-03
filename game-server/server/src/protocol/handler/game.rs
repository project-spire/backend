mod movement_command;

use actix::Actor;
use tracing::error;
use crate::network::session::SessionContext;
use crate::protocol::game::*;
use crate::world::zone::Zone;

pub fn handle(
    zone: &mut Zone,
    ctx: &mut <Zone as Actor>::Context,
    session_ctx: SessionContext,
    proto: GameClientProtocol
) {
    use game_client_protocol::Protocol::*;

    let proto = match proto.protocol {
        Some(p) => p,
        None => {
            error!("Invalid net protocol");
            return;
        },
    };

    match proto {
        MovementCommand(cmd) => movement_command::handle(zone, session_ctx, cmd),
        _ => {
            error!("Unhandled protocol");
        },
    }
}