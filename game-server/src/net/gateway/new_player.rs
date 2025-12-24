use actix::{ActorFutureExt, AsyncContext, Handler, WrapFuture};
use tracing::{error, info};

use super::Gateway;
use crate::net::session::Session;
use crate::net::zone;
use crate::player::PlayerData;
use protocol::game::auth::login;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct NewPlayer {
    pub login_kind: login::Kind,
    pub session: Session,
}

impl Handler<NewPlayer> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: NewPlayer, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(async move {
            let session = msg.session;
            let player_data = match msg.login_kind {
                login::Kind::Enter => PlayerData::load(&session.entry).await?,
                login::Kind::Transfer => todo!(),
            };

            Ok::<(Session, PlayerData), db::Error>((session, player_data))
        }
        .into_actor(self)
        .then(|res, act, _| {
            let (session, player_data) = match res {
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
            default_zone.do_send(zone::PlayerTransfer { session, player_data });

            actix::fut::ready(())
        }));
    }
}
