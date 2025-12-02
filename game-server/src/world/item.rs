use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Item {
    pub data: &'static data::item::Item,
    pub count: u16,
}