use bevy_ecs::component::Component;

#[derive(Component, Debug)]
pub struct Account {
    pub account_id: i64,
}
