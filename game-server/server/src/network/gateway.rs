mod new_player;
mod new_zone;

pub use new_player::NewPlayer;
pub use new_zone::NewZone;

use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use std::collections::HashMap;
use std::sync::Arc;
use crate::database::DatabaseContext;
use crate::world::zone::Zone;

pub struct Gateway {
    zones: HashMap<i64, Addr<Zone>>,

    db_ctx: Arc<DatabaseContext>,
}

impl Gateway {
    pub fn new(db_ctx: Arc<DatabaseContext>) -> Self {
        let zones = HashMap::new();

        Gateway { zones, db_ctx }
    }
}

impl Actor for Gateway {
    type Context = Context<Self>;
}
