use tracing::{error, info};

use crate::handler::ProtocolHandler;
use crate::net::session::SessionContext;
use protocol::game::net::Ping;

impl ProtocolHandler for Ping {
    async fn handle(self, ctx: &SessionContext) {
        info!("Ping: {}", self.timestamp);
    }
}
