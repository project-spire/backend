use std::ops::Deref;

use actix::prelude::*;
use bevy_ecs::prelude::*;

use super::Zone;
use crate::character::status::movement::Movement;
use crate::net::session::SessionContext;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MovementCommand {
    inner: protocol::game::play::MovementCommand,
    ctx: SessionContext,
}

impl Handler<MovementCommand> for Zone {
    type Result = ();

    fn handle(&mut self, msg: MovementCommand, _: &mut Self::Context) -> Self::Result {
        let Some(command) = msg.command else {
            return;
        };
        
        self.with_component_mut(
            msg.ctx.character_id(),
            |movement: Mut<Movement>| {
                todo!("Modify movement.");
            },
        );
    }
}

impl MovementCommand {
    pub fn new(
        inner: protocol::game::play::MovementCommand,
        ctx: SessionContext,
    ) -> Self {
        Self { inner, ctx }
    }
}

impl Deref for MovementCommand {
    type Target = protocol::game::play::MovementCommand;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
