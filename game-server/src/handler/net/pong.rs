use crate::handler::ProtocolGlobalHandler;
use crate::net::session::SessionContext;
use protocol::game::net::Pong;
use tracing::info;

impl ProtocolGlobalHandler for Pong {
    fn handle(self, ctx: SessionContext) {
        info!("{} ponged: {}", ctx, self.timestamp);
    }
}
