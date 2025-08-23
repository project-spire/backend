pub use std::str::FromStr;
use bevy_ecs::component::Component;
use uuid::Uuid;

#[derive(Component, Debug)]
pub struct Account {
    pub account_id: Uuid,
}
