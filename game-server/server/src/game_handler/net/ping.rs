use game_protocol::net::*;
use tracing::error;
use crate::network::session::SessionContext;

pub fn handle(session_ctx: SessionContext, ping: Ping) {
    use net_server_protocol::Protocol;

    let protocol = NetServerProtocol {
        protocol: Some(Protocol::Pong(Pong {timestamp: 1}))
    };
    let buf = match encode(&protocol) {
        Ok(buf) => buf,
        Err(e) => {
            error!("Error encoding pong: {}", e);
            return;
        }
    };

    session_ctx.do_send(buf);
}
