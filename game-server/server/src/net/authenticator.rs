use actix::{Actor, Context, Handler, Message};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aid: String, // account_id
    cid: String, // character_id
    prv: String, // privilege
}

pub struct Authenticator {}

impl Actor for Authenticator {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewConnection {
    pub socket: TcpStream,
}

impl Handler<NewConnection> for Authenticator {
    type Result = ();

    fn handle(&mut self, msg: NewConnection, ctx: &mut Self::Context) -> Self::Result {
        println!("New connection received");
    }
}
