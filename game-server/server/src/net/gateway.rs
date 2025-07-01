use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use crate::net::authenticator::Entry;
use crate::world::zone::Zone;
use protocol::auth::login;
use std::collections::HashMap;
use tokio::net::TcpStream;

pub struct Gateway {
    fields: HashMap<u32, Addr<Zone>>,
}

impl Gateway {
    pub fn new() -> Self {
        let zones = HashMap::new();

        Gateway { fields: zones }
    }
}

impl Actor for Gateway {
    type Context = Context<Self>;
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewSession {
    pub socket: TcpStream,
    pub login_kind: login::Kind,
    pub entry: Entry,
}

impl Handler<NewSession> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewSession, ctx: &mut Self::Context) -> Self::Result {
        let socket = msg.socket;
        let kind = msg.login_kind;

        let future = async move {
            
            
            Ok(socket)
        }
        .into_actor(self)
        .then(|res, act, ctx| {
            actix::fut::ready(())
        });

        ctx.spawn(future);
    }
}