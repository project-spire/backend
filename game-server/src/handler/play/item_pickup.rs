use crate::character::inventory::Inventory;
use crate::handler::ProtocolLocalHandler;
use crate::world::zone::Zone;
use bevy_ecs::prelude::*;
use protocol::game::play::ItemPickup;

impl ProtocolLocalHandler for ItemPickup {
    fn handle(self, entity: Entity, zone: &mut Zone) {
        let Some(mut inventory) = zone.world.get_mut::<Inventory>(entity) else {
            return;
        };
    }
}
