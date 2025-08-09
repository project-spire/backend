mod new_player;

pub use new_player::NewPlayer;

use actix::{Actor, ActorFutureExt, AsyncContext, Context, WrapFuture};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::net::session::IngressProtocol;

const INGRESS_PROTOCOL_BUFFER_SIZE: usize = 256;
const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub struct Zone {
    pub id: i64,

    ingress_proto_tx: mpsc::Sender<IngressProtocol>,
    ingress_proto_rx: Option<mpsc::Receiver<IngressProtocol>>,

    pub world: World,
    pub ticks: u64,
    pub characters: HashMap<Uuid, Entity>,
}

impl Zone {
    pub fn new(id: i64) -> Self {
        let (ingress_proto_tx, ingress_proto_rx) = mpsc::channel(INGRESS_PROTOCOL_BUFFER_SIZE);

        Zone {
            id,
            ingress_proto_tx,
            ingress_proto_rx: Some(ingress_proto_rx),
            world: World::new(),
            ticks: 0,
            characters: HashMap::new(),
        }
    }

    fn handle_protocol_loop(
        &self,
        ctx: &mut <Self as Actor>::Context,
        mut ingress_proto_rx: mpsc::Receiver<IngressProtocol>,
    ) {
        // Start a protocol handling recursion
        ctx.spawn(
            async move {
                let mut protos = Vec::with_capacity(INGRESS_PROTOCOL_BUFFER_SIZE);
                _ = ingress_proto_rx
                    .recv_many(&mut protos, INGRESS_PROTOCOL_BUFFER_SIZE)
                    .await;

                (ingress_proto_rx, protos)
            }
            .into_actor(self)
            .then(|res, act, ctx| {
                let (ingress_proto_rx, mut protos) = res;
                for proto in protos.drain(..) {
                    act.handle_protocol(ctx, proto);
                }

                // Recursion without starving Actor's tick task
                act.handle_protocol_loop(ctx, ingress_proto_rx);

                actix::fut::ready(())
            }),
        );
    }

    fn tick(&mut self, ctx: &mut <Self as Actor>::Context) {
        let mut schedule = Schedule::default();
        schedule.run(&mut self.world);

        self.ticks += 1;
    }
}

impl Actor for Zone {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let ingress_proto_rx = self
            .ingress_proto_rx
            .take()
            .expect("Ingress protocol channel must be set before start");

        self.handle_protocol_loop(ctx, ingress_proto_rx);

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
