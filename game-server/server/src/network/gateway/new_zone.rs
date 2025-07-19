use actix::{Addr, Handler};
use tracing::info;
use crate::network::gateway::Gateway;
use super::Zone;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewZone {
    pub id: i64,
    pub zone: Addr<Zone>,
}

impl Handler<NewZone> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewZone, _: &mut Self::Context) -> Self::Result {
        self.zones.insert(msg.id, msg.zone);
        info!("New zone added: {}", msg.id);
    }
}
