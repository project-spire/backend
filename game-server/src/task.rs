use bevy_ecs::prelude::*;
use futures::future::BoxFuture;
use std::collections::VecDeque;
use std::fmt::Formatter;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::warn;

type CallbackFn = Box<dyn FnOnce(Option<Error>, Entity, &mut World) + Send + Sync>;

#[derive(Component, Default)]
pub struct TaskQueue {
    pub tasks: VecDeque<Task>,
}

pub struct Task {
    policy: Policy,
    state: State,
    callback: Option<CallbackFn>,
}
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

enum State {
    Idle(BoxFuture<'static, Result<(), Error>>),
    Running(tokio::task::JoinHandle<Result<(), Error>>),
    Empty,
}

pub enum Policy {
    /// Run the async task only after the preceding task is done.
    Serial,

    /// Run the async task right away. Callbacks are still synchronized.
    Parallel,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DB(#[from] db::Error),

    #[error("Task execution panicked or failed")]
    Join,
}

impl TaskQueue {
    pub fn dispatch(&mut self, mut task: Task) {
        if let Policy::Parallel = &task.policy {
            task.spawn_if_idle();
        }

        self.tasks.push_back(task);
    }
}

impl Task {
    pub fn new<F>(policy: Policy, future: F) -> Self
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
    {
        Self {
            policy,
            state: State::Idle(Box::pin(future)),
            callback: None,
        }
    }

    pub fn serial<F>(future: F) -> Self
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
    {
        Self::new(Policy::Serial, future)
    }

    pub fn parallel<F>(future: F) -> Self
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
    {
        Self::new(Policy::Parallel, future)
    }

    pub fn on_complete<F>(mut self, callback: F) -> Self
    where
        F: FnOnce(Option<Error>, Entity, &mut World) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    pub fn on_success<F>(mut self, callback: F) -> Self
    where
        F: FnOnce(Entity, &mut World) + Send + Sync +'static,
    {
        self.callback = Some(Box::new(|error, entity, world| {
            if error.is_none() {
                callback(entity, world);
            }
        }));
        self
    }

    fn spawn_if_idle(&mut self) {
        let State::Idle(_) = self.state else {
            return;
        };

        let idle_state = std::mem::replace(&mut self.state, State::Empty);
        let State::Idle(future) = idle_state else {
            unreachable!();
        };

        let handle = tokio::spawn(future);
        self.state = State::Running(handle);
    }
}

impl EntityCommand for Task {
    fn apply(self, mut entity: EntityWorldMut) {
        let mut task_queue = match entity.get_mut::<TaskQueue>() {
            Some(task_queue) => task_queue,
            None => {
                entity.insert(TaskQueue::default());
                entity.get_mut::<TaskQueue>().unwrap()
            }
        };

        task_queue.dispatch(self);
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle(_) => write!(f, "Idle"),
            State::Running(_) => write!(f, "Running"),
            State::Empty => write!(f, "Empty"),
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(process);
}

fn process(world: &mut World) {
    // Collect callbacks. (Need a buffer to limit the scope of `world` borrowing.)
    let callbacks = poll(world);

    for (result, entity, callback) in callbacks {
        callback(result, entity, world);
    }
}

fn poll(world: &mut World) -> Vec<(Option<Error>, Entity, CallbackFn)> {
    let mut callbacks = Vec::new();

    let mut query = world.query::<(Entity, &mut TaskQueue)>();
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    for (entity, mut task_queue) in query.iter_mut(world) {
        loop {
            // 1. Peek the front task and run.
            let Some(task) = task_queue.tasks.front_mut() else {
                break;
            };
            task.spawn_if_idle();

            // 2. Poll the running task.
            let result = if let State::Running(handle) = &mut task.state {
                match Pin::new(handle).poll(&mut cx) {
                    Poll::Ready(result) => result.map_err(|_| Error::Join).and_then(|x| x),
                    Poll::Pending => break,
                }
            } else {
                warn!("Invalid task state ({}) encountered", task.state);
                break;
            };

            // 3. Collect callbacks of the complete task.
            if let Some(mut task) = task_queue.tasks.pop_front() {
                if let Some(callback) = task.callback.take() {
                    callbacks.push((result.err(), entity, callback));
                }
            }
        }
    }

    callbacks
}
