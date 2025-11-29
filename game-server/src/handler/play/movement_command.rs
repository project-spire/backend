use crate::handler::ProtocolHandler;
use crate::net::session::SessionContext;
use crate::world::zone::{self};
use protocol::game::play::MovementCommand;

impl ProtocolHandler for MovementCommand {
    async fn handle(self, ctx: &SessionContext) {
        ctx.do_send_to_zone(zone::MovementCommand::new(self, ctx.clone())).await;
    }
}
