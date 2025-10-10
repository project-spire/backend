use tracing::{error, info};

use crate::net::session::SessionContext;
use protocol::game::net::*;

pub fn handle(session_ctx: SessionContext, ping: &Ping) {
    info!("Ping: {}", ping.timestamp);
}
