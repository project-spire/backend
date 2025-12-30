use crate::character::inventory::Inventory;
use crate::handler::ProtocolLocalHandler;
use crate::net::session::SessionContext;
use bevy_ecs::prelude::*;
use protocol::game::play::ItemPickup;

impl ProtocolLocalHandler for ItemPickup {
    fn handle(self, world: &mut World, entity: Entity, _: SessionContext) {
        let Some(mut inventory) = world.get_mut::<Inventory>(entity) else {
            return;
        };
    }
}
