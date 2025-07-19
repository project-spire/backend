pub use std::str::FromStr;
use bevy_ecs::component::Component;
use strum::EnumString;

#[derive(Clone, Copy, Debug, EnumString)]
pub enum Privilege {
    None,
    Manager,
}

#[derive(Component, Debug)]
pub struct Account {
    pub account_id: i64,
    pub privilege: Privilege,
}
