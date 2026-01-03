use crate::handler::ProtocolGlobalHandler;
use crate::net::session::Session;
use protocol::game::net::Pong;
use tracing::info;

impl ProtocolGlobalHandler for Pong {
    fn handle(self, session: Session) {
        info!("{} ponged: {}", session, self.timestamp);
    }
}
