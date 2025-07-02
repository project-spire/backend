use actix::{Actor, ActorFutureExt, AsyncContext, Context, WrapFuture};
use crate::net::session::IngressMessage;
use std::time::Duration;
use tokio::sync::mpsc;

const INGRESS_MESSAGE_BUFFER_SIZE: usize = 64;
const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub struct Zone {
    ingress_msg_tx: mpsc::Sender<IngressMessage>,
    ingress_msg_rx: Option<mpsc::Receiver<IngressMessage>>,
    ticks: u64,
}

impl Zone {
    pub fn new() -> Self {
        let (ingress_msg_tx, ingress_msg_rx) = mpsc::channel(INGRESS_MESSAGE_BUFFER_SIZE);

        Zone {
            ingress_msg_tx,
            ingress_msg_rx: Some(ingress_msg_rx),
            ticks: 0,
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
