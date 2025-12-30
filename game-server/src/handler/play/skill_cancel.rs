use crate::handler::ProtocolLocalHandler;
use crate::net::session::SessionContext;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::World;
use protocol::game::play::SkillCancel;

impl ProtocolLocalHandler for SkillCancel {
    fn handle(self, world: &mut World, entity: Entity, ctx: SessionContext) {
        todo!()
    }
}