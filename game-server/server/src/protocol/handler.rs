pub mod net;
pub mod game;

use actix::Actor;
use tracing::error;
use crate::network::session::IngressProtocol;
use crate::protocol::{*, net::NetClientProtocol, game::GameClientProtocol};
use crate::world::zone::Zone;

pub fn handle(zone: &mut Zone, ctx: &mut <Zone as Actor>::Context, proto: IngressProtocol) {
    let (session_ctx, category, data) = proto;

    match category {
        Category::Auth => todo!(),
        Category::Game => {
            match GameClientProtocol::decode(data) {
                Ok(proto) => {
                    game::handle(zone, ctx, session_ctx, proto)
                },
                Err(e) => {
                    error!("Error decoding NetClientProtocol: {:?}", e);
                }
            }
        },
        Category::Net => {
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
