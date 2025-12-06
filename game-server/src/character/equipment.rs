use bevy_ecs::prelude::*;

use common::id::Id;

#[derive(Component)]
pub struct Equipment {
    pub layout: Layout,
}

pub struct EquipmentItem {
    pub id: Id,
    pub data: &'static data::item::Equipment,
    pub count: u16,
}

pub enum Layout {
    Humanoid {
        items: [Option<EquipmentItem>; HumanoidSlot::Size as usize],
    },
    Riding {
        items: [Option<EquipmentItem>; RidingSlot::Size as usize],
    },
}

pub enum HumanoidSlot {
    Head = 0,
    Chest = 1,

    Size = 2,
}

pub enum RidingSlot {
    Head = 0,
    Body = 1,

    Size = 2,
}
