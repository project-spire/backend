use actix::{Actor, ActorFutureExt, AsyncContext, Context, WrapFuture};
use tokio::sync::mpsc;

const IN_MESSAGE_BUFFER_SIZE: usize = 32;

pub struct Zone {
    in_message_tx: mpsc::Sender<()>,
    in_message_rx: Option<mpsc::Receiver<()>>,
}

impl Zone {
    pub fn new() -> Self {
        let (in_message_tx, in_message_rx) = mpsc::channel(IN_MESSAGE_BUFFER_SIZE);
        let in_message_rx = Some(in_message_rx);

        Zone {
            in_message_tx,
            in_message_rx,
        }
    }

    fn handle_in_messages(
        &self,
        ctx: &mut <Self as Actor>::Context,
        mut in_message_rx: mpsc::Receiver<()>,
    ) {
        // Start a message handling recursion
        ctx.spawn(
            async move {
                let mut in_messages = Vec::with_capacity(IN_MESSAGE_BUFFER_SIZE);
                _ = in_message_rx
                    .recv_many(&mut in_messages, IN_MESSAGE_BUFFER_SIZE)
                    .await;

                (in_message_rx, in_messages)
            }
            .into_actor(self)
            .then(|res, act, ctx| {
                let (in_message_rx, mut in_messages) = res;
                for in_message in in_messages.drain(..) {
                    act.handle_in_message(ctx, in_message);
                }

                // Recursion without starving Actor's update task
                act.handle_in_messages(ctx, in_message_rx);

                actix::fut::ready(())
            }),
        );
    }

    fn handle_in_message(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        in_message: (),
    ) {

    }
}

impl Actor for Zone {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let in_message_rx = self
            .in_message_rx
            .take()
            .expect("InMessage channel should be set before start");

        self.handle_in_messages(ctx, in_message_rx);
    }
}
