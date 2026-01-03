use crate::handler::ProtocolGlobalHandler;
use crate::net::session::Session;
use protocol::game::net::Ping;
use tracing::info;

impl ProtocolGlobalHandler for Ping {
    fn handle(self, session: Session) {
        info!("{} pinged: {}", session, self.timestamp);
    }
}
