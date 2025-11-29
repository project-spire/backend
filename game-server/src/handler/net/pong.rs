use tracing::{error, info};

use crate::handler::ProtocolHandler;
use crate::net::session::SessionContext;
use protocol::game::net::Pong;

impl ProtocolHandler for Pong {
    async fn handle(self, ctx: &SessionContext) {
        info!("Pong: {}", self.timestamp);
    }
}
