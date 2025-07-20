mod ping;

use actix::Actor;
use game_protocol::net::*;
use tracing::error;
use crate::network::session::SessionContext;
use crate::world::zone::Zone;

pub fn handle(
    zone: &mut Zone,
    ctx: &mut <Zone as Actor>::Context,
    session_ctx: SessionContext,
    proto: NetClientProtocol
) {
    use net_client_protocol::Protocol;

    let proto = match proto.protocol {
        Some(p) => p,
        None => {
            error!("Invalid net protocol");
            return;
        },
    };

    match proto {
        Protocol::Ping(ping) => ping::handle(session_ctx, ping),
        _ => {
            error!("Unhandled protocol");
        },
    }
}