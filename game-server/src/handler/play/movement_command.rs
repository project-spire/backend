use crate::handler::ProtocolLocalHandler;
use crate::world::zone::Zone;
use protocol::game::play::MovementCommand;

impl ProtocolLocalHandler for MovementCommand {
    fn handle(self, zone: &mut Zone) {
        unimplemented!();
    }
}
