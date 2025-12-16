use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::World;
use crate::handler::ProtocolLocalHandler;
use protocol::game::play::SkillCancel;

impl ProtocolLocalHandler for SkillCancel {
    fn handle(self, world: &mut World, entity: Entity) {
        todo!()
    }
}