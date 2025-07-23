mod net;
mod game;

use actix::Actor;
use game_protocol::{*, net::NetClientProtocol, game::GameClientProtocol};
use tracing::error;
use crate::network::session::IngressProtocol;
use crate::world::zone::Zone;

pub fn handle(zone: &mut Zone, ctx: &mut <Zone as Actor>::Context, proto: IngressProtocol) {
    let (session_ctx, category, data) = proto;

    match category {
        ProtocolCategory::Auth => todo!(),
        ProtocolCategory::Game => {
            match GameClientProtocol::decode(data) {
                Ok(proto) => {
                    game::handle(zone, ctx, session_ctx, proto)
                },
                Err(e) => {
                    error!("Error decoding NetClientProtocol: {:?}", e);
                }
            }
        },
        ProtocolCategory::Net => {
            match NetClientProtocol::decode(data) {
                Ok(proto) => {
                    net::handle(zone, ctx, session_ctx, proto)
                },
                Err(e) => {
                    error!("Error decoding NetClientProtocol: {:?}", e);
                }
            }
        },
    };
}
