use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use crate::database::{DatabaseClient, DatabaseContext};
use crate::network::authenticator::Entry;
use crate::player::PlayerData;
use crate::world::zone::Zone;
use protocol::auth::login;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;

pub struct Gateway {
    zones: HashMap<u32, Addr<Zone>>,

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

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub socket: TcpStream,
    pub login_kind: login::Kind,
    pub entry: Entry,
}

impl Handler<NewPlayer> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        let db_ctx = self.db_ctx.clone();

        ctx.spawn(async move {
            let db_client = db_ctx.client().await?;
            let player_data = match msg.login_kind {
                login::Kind::Enter => PlayerData::load(&db_client, &msg.entry).await?,
                login::Kind::Transfer => todo!(),
            };

            Ok::<PlayerData, Box<dyn std::error::Error>>(player_data)
        }
        .into_actor(self)
        .then(|res, _, _| {
            match res {
                Ok(data) => {
                    println!("Player loaded: {:?}, {:?}", data.account, data.character );
                },
                Err(e) => {
                    eprintln!("Error loading player: {}", e);
                }
            }

            actix::fut::ready(())
        }));
    }
}
