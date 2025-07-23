use game_protocol::game::*;
use tracing::error;
use crate::character::movement::Movement;
use crate::network::session::SessionContext;
use crate::timestamp::Timestamp;
use crate::world::zone::Zone;

pub fn handle(zone: &mut Zone, session_ctx: SessionContext, proto: MovementCommandProtocol) {
    if proto.command.is_none() {
        error!("Empty command");
        return;
    }

    let entity = match zone.characters.get(&session_ctx.entry.character_id) {
        Some(e) => e,
        None => {
            error!("{} Could not find character", session_ctx);
            return;
        }
    };
    let mut entity = match zone.world.get_entity_mut(*entity) {
        Ok(e) => e,
        Err(e) => {
            error!("{} Could not find entity: {}", session_ctx, e);
            return;
        }
    };

    let mut movement = match entity.get_mut::<Movement>() {
        Some(t) => t,
        None => {
            error!("{} Could not find movement", session_ctx);
            return;
        }
    };

    movement.add_command(Timestamp::from_millis(proto.timestamp), proto.command.unwrap().into());
}
