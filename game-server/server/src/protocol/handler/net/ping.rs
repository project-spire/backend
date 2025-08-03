use tracing::error;
use crate::network::session::SessionContext;
use crate::protocol::net::*;

pub fn handle(session_ctx: SessionContext, ping: PingProtocol) {
    use net_server_protocol::Protocol;

    let protocol = NetServerProtocol {
        protocol: Some(Protocol::Pong(PongProtocol {timestamp: 1}))
    };
    let buf = match encode_net(&protocol) {
        Ok(buf) => buf,
        Err(e) => {
            error!("Error encoding pong: {}", e);
            return;
        }
    };

    session_ctx.do_send(buf);
}
