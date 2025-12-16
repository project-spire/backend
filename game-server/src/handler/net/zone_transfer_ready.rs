use crate::handler::ProtocolGlobalHandler;
use crate::net::session::Entry;
use protocol::game::net::ZoneTransferReady;

impl ProtocolGlobalHandler for ZoneTransferReady {
    fn handle(self, entry: Entry) {
        todo!()
    }
}