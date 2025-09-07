mod new_player;
mod new_zone;

pub use new_player::NewPlayer;
pub use new_zone::NewZone;

use actix::prelude::*;
use std::collections::HashMap;
use crate::db;
use crate::world::zone::Zone;

pub struct Gateway {
    zones: HashMap<i64, Addr<Zone>>,

    db_pool: db::Pool,
}

impl Gateway {
    pub fn new(db_pool: db::Pool) -> Self {
        let zones = HashMap::new();

        Gateway { zones, db_pool }
    }
}

impl Actor for Gateway {
    type Context = Context<Self>;
}
