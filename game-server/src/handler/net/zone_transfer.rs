use crate::handler::ProtocolGlobalHandler;
use crate::net::session::Entry;
use protocol::game::net::ZoneTransfer;

impl ProtocolGlobalHandler for ZoneTransfer {
    fn handle(self, entry: Entry) {
        todo!()
    }
}