mod new_player;
mod new_zone;

pub use new_player::NewPlayer;
pub use new_zone::NewZone;

use crate::db;
use crate::world::zone::Zone;
use actix::prelude::*;
use std::collections::HashMap;

pub struct Gateway {
    zones: HashMap<i64, Addr<Zone>>,
}

impl Default for Gateway {
    fn default() -> Self {
        Self {
            zones: HashMap::new(),
        }
    }
}

impl Actor for Gateway {
    type Context = Context<Self>;
}

impl Supervised for Gateway {}

impl SystemService for Gateway {}
