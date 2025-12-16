use crate::handler::ProtocolLocalHandler;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::World;
use protocol::game::play::SkillUse;

impl ProtocolLocalHandler for SkillUse {
    fn handle(self, world: &mut World, entity: Entity) {
        todo!()
    }
}