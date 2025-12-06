use bevy_ecs::prelude::*;
use futures::future::BoxFuture;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

pub type TaskFuture = BoxFuture<'static, Result<(), Error>>;
pub type CallbackFn = Box<dyn FnOnce(Option<Error>) + Send + Sync>;

#[derive(Component, Default)]
pub struct Transactions {
    pub queue: VecDeque<Transaction>,
}

pub struct Transaction {
    state: State,
    callback: Option<CallbackFn>,
}
unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

pub enum State {
    Idle(TaskFuture),
    Running(tokio::task::JoinHandle<Result<(), Error>>),
    Empty,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DB(#[from] db::Error),

    #[error("Task execution panicked or failed")]
    Join,
}

impl Transactions {
    pub fn post(
        &mut self,
        task: impl Future<Output = Result<(), Error>> + Send + 'static,
        callback: Option<Box<dyn FnOnce(Option<Error>) + Send + Sync + 'static>>,
    ) {
        self.queue.push_back(Transaction {
            state: State::Idle(Box::pin(task)),
            callback,
        });
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(process);
}

fn process(mut query: Query<(Entity, &mut Transactions)>) {
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    for (entity, mut transactions) in query.iter_mut() {
        // 1. Peek at the front of the queue.
        let state = match transactions.queue.front_mut() {
            Some(tx) => &mut tx.state,
            None => continue,
        };

        // 2. Run the task if idle.
        if let State::Idle(_) = state {
            let idle_state = std::mem::replace(state, State::Empty);

            if let State::Idle(future) = idle_state {
                let handle = tokio::spawn(future);
                *state = State::Running(handle);

                continue;
            } else {
                unreachable!();
            }
        }

        // 3. Poll the running task.
        let result = if let State::Running(handle) = state {
            match Pin::new(handle).poll(&mut cx) {
                Poll::Ready(result) => result.map_err(|_| Error::Join).and_then(|x| x),
                Poll::Pending => continue,
            }
        } else {
            // Impossible state
            continue;
        };

        // 4. Callback of the complete task.
        let tx_finished = transactions.queue.pop_front().unwrap();
        if let Some(callback) = tx_finished.callback {
            callback(result.err());
        }
    }
}
