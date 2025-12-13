use crate::calc::BasedValue;
use crate::physics::Speed;
use crate::world::time::Time;
use crate::world::transform::Transform;
use bevy_ecs::prelude::*;
use nalgebra::{UnitVector2, Vector2};
use tracing::warn;
use protocol::game::play::{movement_command, movement_command::Command::*, MovementCommand};
use util::timestamp::Timestamp;

const COMMANDS_BUFFER_SIZE: usize = 4;

#[derive(Component)]
pub struct Movement {
    pub state: State,
    pub motion: Motion,
    pub direction: UnitVector2<f32>,

    pub commands: Vec<MovementCommand>,
    last_timestamp: Timestamp,

    pub walk_speed: BasedValue<Speed>,
    pub run_speed: BasedValue<Speed>,
}

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Normal,
    Bound,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Motion {
    #[default]
    Idle,
    Walking,
    Running,
    Rolling,
    Jumping,
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

impl Default for Movement {
    fn default() -> Self {
        Self {
            state: State::default(),
            motion: Motion::default(),
            direction: UnitVector2::new_normalize(Vector2::new(1.0, 0.0)),
            commands: Vec::with_capacity(COMMANDS_BUFFER_SIZE),
            last_timestamp: Timestamp::default(),
            walk_speed: BasedValue::new(Speed::default()),
            run_speed: BasedValue::new(Speed::default()),
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(handle_commands);
}

fn handle_commands(
    mut query: Query<(&mut Movement, &mut Transform)>,
    time: Res<Time>,
) {
    let now = util::timestamp::now();
    let mut commands = Vec::with_capacity(COMMANDS_BUFFER_SIZE * 2);

    for (mut movement, mut transform) in query.iter_mut() {
        if movement.commands.is_empty() {
            continue;
        }

        for command in movement.commands.drain(..) {
            let timestamp = command.timestamp;
            let Some(command) = command.command else {
                continue;
            };

            commands.push((timestamp, command));
        }
        movement.commands.shrink_to(COMMANDS_BUFFER_SIZE);

        for i in 0..commands.len() {
            let (timestamp, command) = commands.get(i).unwrap();
            let next_timestamp = match commands.get(i + 1) {
                Some((next_timestamp, _)) => *next_timestamp,
                None => now,
            };

            if next_timestamp <= *timestamp || *timestamp < movement.last_timestamp {
                warn!("Invalid movement timestamp");
                continue;
            }
            let dt = (next_timestamp - timestamp) as f32;

            match command {
                Walk(walk) => handle_walk(&mut movement, &mut transform, dt, &walk),
                Run(run) => handle_run(&mut movement, &mut transform, dt, &run),
                Roll(roll) => handle_roll(&mut movement, dt, &roll),
                Jump(jump) => handle_jump(&mut movement, dt, &jump),
            }
        }
        movement.last_timestamp = now;

        commands.clear();
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

    movement.motion = Motion::Walking;
    movement.direction = direction;
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
    movement.direction = direction;
}

fn handle_roll(
    movement: &mut Movement,
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
    movement.direction = direction;
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
