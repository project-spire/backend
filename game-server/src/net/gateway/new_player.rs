use actix::{ActorFutureExt, AsyncContext, Handler, WrapFuture};
use quinn::{Connection, RecvStream};
use tracing::{error, info};
use crate::net::session::Entry;
use crate::player::PlayerData;
use crate::protocol::game::auth::login;
use crate::world::zone;
use super::Gateway;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub connection: Connection,
    pub login_kind: login::Kind,
    pub entry: Entry,
}

impl Handler<NewPlayer> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.db_pool.clone();

        ctx.spawn(async move {
            let mut tx = db_pool.begin().await?;

            let player_data = match msg.login_kind {
                login::Kind::Enter => PlayerData::load(&mut tx, &msg.entry).await?,
                login::Kind::Transfer => todo!(),
            };

            tx.commit().await?;

            Ok::<(Entry, Connection, PlayerData), Box<dyn std::error::Error>>((
                msg.entry,
                msg.connection,
                player_data
            ))
        }
        .into_actor(self)
        .then(|res, act, _| {
            if let Err(e) = res {
                error!("Failed to load player data: {}", e);
                return actix::fut::ready(());
            }

            let (entry, connection, player_data) = res.unwrap();
            info!("Player loaded: {:?}, {:?}", player_data.account, player_data.character );

            //TODO: Find the player's last zone
            let default_zone = act.zones.get(&0).unwrap();
            default_zone.do_send(zone::NewPlayer::new(entry, connection, player_data));

            actix::fut::ready(())
        }));
    }
}