use crate::calc::BasedValue;
use crate::net::session::Session;
use crate::physics::Speed;
use crate::world::transform::Transform;
use bevy_ecs::prelude::*;
use nalgebra::UnitVector2;
use protocol::game::encode;
use protocol::game::play::movement_command::{self, Command::*};
use protocol::game::play::{MovementCommand, MovementState, MovementSync};
use protocol::game::play::movement_state::Motion;
use tracing::warn;
use util::timestamp::Timestamp;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Movement {
    pub state: State,
    pub motion: Motion,

    pub walk_speed: BasedValue<Speed>,
    pub run_speed: BasedValue<Speed>,
}

#[derive(Component, Default)]
#[require(Movement)]
pub struct MovementCommands {
    pub queue: Vec<MovementCommand>,
    last_timestamp: Timestamp,
}

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Normal,
    Bound,
}

impl Movement {
    pub fn can_move(&self) -> bool {
        match self.state {
            State::Normal => true,
            State::Bound => false,
        }
    }

    pub fn can_jump(&self) -> bool {
        if !self.can_move() {
            return false;
        }

        //TODO: Check if is on the ground

        true
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        process_commands,
        sync_movement_states,
    ).chain());
}

fn process_commands(
    mut query: Query<(&mut MovementCommands, &mut Movement, &mut Transform)>,
) {
    let now = util::timestamp::now();
    let mut commands_buffer = Vec::with_capacity(8);

    for (mut commands, mut movement, mut transform) in query.iter_mut() {
        if commands.queue.is_empty() {
            continue;
        }

        for command in commands.queue.drain(..) {
            let timestamp = command.timestamp;
            let Some(command) = command.command else {
                continue;
            };

            commands_buffer.push((timestamp, command));
        }

        for i in 0..commands_buffer.len() {
            let (timestamp, command) = commands_buffer.get(i).unwrap();
            let next_timestamp = match commands_buffer.get(i + 1) {
                Some((next_timestamp, _)) => *next_timestamp,
                None => now,
            };

            if next_timestamp <= *timestamp || *timestamp < commands.last_timestamp {
                warn!("Invalid movement timestamp");
                continue;
            }
            let dt = (next_timestamp - timestamp) as f32;

            match command {
                Walk(walk) => handle_walk(&mut movement, &mut transform, dt, &walk),
                Run(run) => handle_run(&mut movement, &mut transform, dt, &run),
                Roll(roll) => handle_roll(&mut movement, &mut transform, dt, &roll),
                Jump(jump) => handle_jump(&mut movement, dt, &jump),
            }
        }
        commands.last_timestamp = now;

        commands_buffer.clear();
    }
}

fn handle_walk(
    movement: &mut Movement,
    transform: &mut Transform,
    dt: f32,
    walk: &movement_command::Walk,
) {
    if !movement.can_move() {
        return;
    }

    let Some::<UnitVector2<f32>>(direction) = walk.direction.and_then(|d| d.try_into().ok()) else {
        return;
    };

    let distance = *movement.walk_speed * dt;
    let dx = direction.x * distance;
    let dz = direction.y * distance;

    // TODO: Check if movable to the target position
    transform.position.x += dx;
    transform.position.z += dz;
    transform.direction = direction;

    movement.motion = Motion::Walking;
}

fn handle_run(
    movement: &mut Movement,
    transform: &mut Transform,
    dt: f32,
    run: &movement_command::Run,
) {
    if !movement.can_move() {
        return;
    }

    let Some(direction) = run.direction.and_then(|d| d.try_into().ok()) else {
        return;
    };

    movement.motion = Motion::Running;
    transform.direction = direction;
}

fn handle_roll(
    movement: &mut Movement,
    transform: &mut Transform,
    dt: f32,
    roll: &movement_command::Roll,
) {
    if !movement.can_move() {
        return;
    }

    let Some(direction) = roll.direction.and_then(|d| d.try_into().ok()) else {
        return;
    };

    movement.motion = Motion::Rolling;
    transform.direction = direction;
}

fn handle_jump(
    movement: &mut Movement,
    dt: f32,
    _: &movement_command::Jump,
) {
    if !movement.can_jump() {
        return;
    }

    movement.motion = Motion::Jumping;
}

fn sync_movement_states(
    query: Query<(Entity, Ref<Movement>, Ref<Transform>)>,
    sessions: Query<&Session>,
) {
    let mut sync = MovementSync::default();
    sync.timestamp = util::timestamp::now();

    for (entity, movement, transform) in query.iter() {
        if !movement.is_changed() && transform.is_changed() {
            continue;
        }

        let state = MovementState {
            entity: entity.to_bits(),
            motion: movement.motion.into(),
            position: Some(transform.position.into()),
            direction: Some(transform.direction.into()),
        };
        sync.states.push(state);
    }

    if sync.states.is_empty() {
        return;
    }

    // TODO: Optimize by sending only if visible
    let Ok(protocol) = encode(&sync) else {
        return;
    };
    for session in sessions.iter() {
        // TODO: Send as datagram?
        _ = session.egress_protocol_sender.send(protocol.clone());
    }
}
