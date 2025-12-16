use crate::handler::ProtocolGlobalHandler;
use crate::net::session::Entry;
use protocol::game::net::Ping;
use tracing::info;

impl ProtocolGlobalHandler for Ping {
    fn handle(self, entry: Entry) {
        info!("[{:?}] Ping: {}", entry, self.timestamp);
    }
}
