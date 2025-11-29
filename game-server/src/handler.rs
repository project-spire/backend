include!(concat!(env!("OUT_DIR"), "/spire.protocol.game.handle.rs"));

use crate::net::session::SessionContext;

pub mod net;
pub mod play;

pub trait ProtocolHandler {
    fn handle(self, ctx: &SessionContext);
}
