mod new_player;
mod new_zone;

pub use new_player::NewPlayer;
pub use new_zone::NewZone;

use crate::net::zone::Zone;
use actix::prelude::*;
use std::collections::HashMap;
use util::id::Id;

#[derive(Default)]
pub struct Gateway {
    zones: HashMap<Id, Addr<Zone>>,
    character_zones: HashMap<Id, Id>,
}

impl Actor for Gateway {
    type Context = Context<Self>;
}

impl Supervised for Gateway {}

impl SystemService for Gateway {}
