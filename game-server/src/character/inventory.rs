use std::collections::HashMap;

use bevy_ecs::prelude::*;

use util::id::Id;

#[derive(Component)]
pub struct Inventory {
    pub items: HashMap<Id, InventoryItem>,

    pub weight_max: u32,
    pub weight_current: u32,
}

pub struct InventoryItem {
    pub id: Id,
    pub data: &'static data::item::Item,
    pub count: u16,
}

impl Inventory {
    pub fn insert_item(&mut self, item: InventoryItem) {
        self.items.insert(item.id, item);
    }

    pub fn remove_item(&mut self, id: Id) -> Option<InventoryItem> {
        self.items.remove(&id)
    }
}
