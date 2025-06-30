use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use crate::world::field::Field;
use protocol::auth::login;
use std::collections::HashMap;
use tokio::net::TcpStream;

pub struct Gateway {
    fields: HashMap<u32, Addr<Field>>,
}

impl Gateway {
    pub fn new() -> Self {
        let fields = HashMap::new();

        Gateway { fields }
    }
}

impl Actor for Gateway {
    type Context = Context<Self>;
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewSession {
    pub socket: TcpStream,
    pub kind: login::Kind,
}

impl Handler<NewSession> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewSession, ctx: &mut Self::Context) -> Self::Result {
        let socket = msg.socket;
        let kind = msg.kind;

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