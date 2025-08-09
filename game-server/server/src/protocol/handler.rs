pub mod net;
pub mod play;

use actix::Actor;
use tracing::error;
use crate::net::session::IngressProtocol;
use crate::protocol::Protocol;
use crate::world::zone::Zone;

impl Zone {
    pub fn handle_protocol(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        ingress_protocol: IngressProtocol
    ) {
        let (session_ctx, protocol) = ingress_protocol;
        
        match protocol {
            Protocol::MovementCommand(movement_command) => {
                self.handle_movement_command(&session_ctx, &movement_command)
            },
            _ => {
                error!("Unhandled protocol: {:?}", protocol);
            },
        }
    }
}
