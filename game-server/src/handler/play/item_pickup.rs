use crate::character::inventory::Inventory;
use crate::handler::ProtocolLocalHandler;
use bevy_ecs::prelude::*;
use protocol::game::play::ItemPickup;

impl ProtocolLocalHandler for ItemPickup {
    fn handle(self, world: &mut World, entity: Entity) {
        let Some(mut inventory) = world.get_mut::<Inventory>(entity) else {
            return;
        };
    }
}
