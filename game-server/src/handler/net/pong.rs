use crate::handler::ProtocolGlobalHandler;
use crate::net::session::Entry;
use protocol::game::net::Pong;
use tracing::info;

impl ProtocolGlobalHandler for Pong {
    fn handle(self, entry: Entry) {
        info!("[{:?}] Pong: {}", entry, self.timestamp);
    }
}
