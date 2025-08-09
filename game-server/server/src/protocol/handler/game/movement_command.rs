use tracing::error;
use crate::character::movement;
use crate::net::session::SessionContext;
use crate::protocol::play::*;
use crate::timestamp::Timestamp;
use crate::world::zone::Zone;

pub fn handle(zone: &mut Zone, session_ctx: SessionContext, protocol: MovementCommand) {
    let command = match protocol.command {
        Some(c) => c,
        None => {
            error!("Empty command");
            return;
        },
    };

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

    let mut movement = match entity.get_mut::<movement::Movement>() {
        Some(t) => t,
        None => {
            error!("{} Could not find movement", session_ctx);
            return;
        }
    };
    
    if let Ok(command) = movement::MovementCommand::try_from(command) {
        movement.add_command(
            Timestamp::from_millis(protocol.timestamp),
            command,
        );
    } else {
        todo!()
    }
}
