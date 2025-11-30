use actix::{ActorFutureExt, AsyncContext, Handler, WrapFuture};
use quinn::{Connection, RecvStream, SendStream};
use tracing::{error, info};

use super::Gateway;
use crate::net::session::{Entry, Session};
use crate::player::PlayerData;
use crate::world::zone;
use protocol::game::auth::login;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub connection: Connection,
    pub receive_stream: RecvStream,
    pub send_stream: SendStream,
    pub login_kind: login::Kind,
    pub entry: Entry,
}

impl Handler<NewPlayer> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(async move {
            let session = Session::start(msg.entry, msg.connection, msg.receive_stream, msg.send_stream);

            let player_data = match msg.login_kind {
                login::Kind::Enter => PlayerData::load(session).await?,
                login::Kind::Transfer => todo!(),
            };

            Ok::<PlayerData, Box<dyn std::error::Error>>(player_data)
        }
        .into_actor(self)
        .then(|res, act, _| {
            let player_data = match res {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to load player data: {}", e);
                    return actix::fut::ready(());
                }
            };
            // info!(
            //     "Player loaded: {:?}, {:?}",
            //     player_data.account, player_data.character
            // );

            //TODO: Find the player's last zone
            let default_zone = act.zones.get(&0).unwrap();
            default_zone.do_send(zone::NewPlayer { player_data });

            actix::fut::ready(())
        }));
    }
}
