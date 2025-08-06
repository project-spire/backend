pub mod net;
pub mod game;

use crate::net::session::IngressProtocol;

pub trait Handler {
    fn handle(&mut self, protocol: IngressProtocol);
}
