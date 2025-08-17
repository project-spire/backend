use actix::{ActorFutureExt, AsyncContext, Handler, WrapFuture};
use quinn::{Connection, SendStream, RecvStream};
use tracing::{error, info};
use crate::net::session::Entry;
use crate::player::PlayerData;
use crate::protocol::auth::login;
use crate::world::zone;
use super::Gateway;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub connection: Connection,
    pub streams: (SendStream, RecvStream),
    pub login_kind: login::Kind,
    pub entry: Entry,
}

impl Handler<NewPlayer> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        let db_ctx = self.db_ctx.clone();

        ctx.spawn(async move {
            let player_data = match msg.login_kind {
                login::Kind::Enter => PlayerData::load(&db_ctx.client, &msg.entry).await?,
                login::Kind::Transfer => todo!(),
            };

            Ok::<(Entry, Connection, (SendStream, RecvStream), PlayerData), Box<dyn std::error::Error>>((
                msg.entry,
                msg.connection,
                msg.streams,
                player_data
            ))
        }
        .into_actor(self)
        .then(|res, act, _| {
            if let Err(e) = res {
                error!("Failed to load player data: {}", e);
                return actix::fut::ready(());
            }

            let (entry, connection, streams, player_data) = res.unwrap();
            info!("Player loaded: {:?}, {:?}", player_data.account, player_data.character );

            //TODO: Find the player's last zone
            let default_zone = act.zones.get(&0).unwrap();
            default_zone.do_send(zone::NewPlayer::new(entry, connection, streams, player_data));

            actix::fut::ready(())
        }));
    }
}