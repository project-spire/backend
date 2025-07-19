mod new_player;

use std::collections::HashMap;
pub use new_player::NewPlayer;

use actix::{Actor, ActorFutureExt, AsyncContext, Context, WrapFuture};
use bevy_ecs::prelude::*;
use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::network::session::IngressMessage;

const INGRESS_MESSAGE_BUFFER_SIZE: usize = 64;
const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub struct Zone {
    id: i64,

    ingress_msg_tx: mpsc::Sender<IngressMessage>,
    ingress_msg_rx: Option<mpsc::Receiver<IngressMessage>>,

    world: World,
    ticks: u64,
    players: HashMap<i64, Entity>,
}

impl Zone {
    pub fn new(id: i64) -> Self {
        let (ingress_msg_tx, ingress_msg_rx) = mpsc::channel(INGRESS_MESSAGE_BUFFER_SIZE);

        Zone {
            id,
            ingress_msg_tx,
            ingress_msg_rx: Some(ingress_msg_rx),
            world: World::new(),
            ticks: 0,
            players: HashMap::new(),
        }
    }

    fn handle_ingress_messages(
        &self,
        ctx: &mut <Self as Actor>::Context,
        mut ingress_msg_rx: mpsc::Receiver<IngressMessage>,
    ) {
        // Start a message handling recursion
        ctx.spawn(
            async move {
                let mut ingress_msg_buf = Vec::with_capacity(INGRESS_MESSAGE_BUFFER_SIZE);
                _ = ingress_msg_rx
                    .recv_many(&mut ingress_msg_buf, INGRESS_MESSAGE_BUFFER_SIZE)
                    .await;

                (ingress_msg_rx, ingress_msg_buf)
            }
            .into_actor(self)
            .then(|res, act, ctx| {
                let (in_message_rx, mut msg_buf) = res;
                for msg in msg_buf.drain(..) {
                    act.handle_ingress_message(ctx, msg);
                }

                // Recursion without starving Actor's tick task
                act.handle_ingress_messages(ctx, in_message_rx);

                actix::fut::ready(())
            }),
        );
    }

    fn handle_ingress_message(&mut self, ctx: &mut <Self as Actor>::Context, msg: IngressMessage) {}

    fn tick(&mut self, ctx: &mut <Self as Actor>::Context) {
        let mut schedule = Schedule::default();
        schedule.run(&mut self.world);

        self.ticks += 1;
    }
}

impl Actor for Zone {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let ingress_msg_rx = self
            .ingress_msg_rx
            .take()
            .expect("InMessage channel should be set before start");

        self.handle_ingress_messages(ctx, ingress_msg_rx);

        ctx.run_interval(TICK_INTERVAL, |act, ctx| {
            act.tick(ctx);
        });
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Zone[{}]", self.id)
    }
}
