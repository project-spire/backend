use crate::handler::ProtocolGlobalHandler;
use crate::net::session::SessionContext;
use protocol::game::net::Ping;
use tracing::info;

impl ProtocolGlobalHandler for Ping {
    fn handle(self, ctx: SessionContext) {
        info!("{} pinged: {}", ctx, self.timestamp);
    }
}
