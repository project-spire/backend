use bevy_ecs::prelude::*;
use nalgebra::{UnitVector2, Vector3};
use tracing::error;
use crate::net::session::{SessionContext};
use crate::protocol;
use crate::protocol::convert::*;
use crate::protocol::play::{movement_command, MovementSync, MovementState, movement_state::Motion};
use crate::util::timestamp::Timestamp;
use crate::world::transform::Transform;

#[derive(Component, Default)]
pub struct Movement {
    motion: Motion,
    direction: Option<UnitVector2<f32>>,
    commands: Vec<(Timestamp, MovementCommand)>,
    stat: MovementStat,
}

pub enum MovementCommand {
    Halt,
    Walk { direction: UnitVector2<f32> },
    Run { direction: UnitVector2<f32> },
    Roll { direction: UnitVector2<f32> },
    Jump,
}

#[derive(Default)]
pub struct MovementStat {

}

impl Movement {
    pub fn add_command(&mut self, timestamp: Timestamp, command: MovementCommand) {
        self.commands.push((timestamp, command));
    }
}

pub fn update(
    mut query: Query<(&mut Movement, &mut Transform)>,
) {
    query.iter_mut().for_each(|(mut movement, mut transform)| {
        let commands: Vec<_> = movement.commands.drain(..).collect();
        for (timestamp, command) in commands {
            handle_command(timestamp, command, &mut movement, &mut transform);
        }

        handle_movement(&movement, &mut transform);
    })
}

fn handle_command(
    timestamp: Timestamp,
    command: MovementCommand,
    movement: &mut Movement,
    transform: &mut Transform,
) {
    use MovementCommand::*;

    match command {
        Halt => {
            movement.motion = Motion::Idle;
            movement.direction = None;
        },
        Walk { direction } => {
            movement.motion = Motion::Walking;
            movement.direction = Some(direction);
            transform.direction = direction;
        }
        Run { direction } => {
            movement.motion = Motion::Running;
            movement.direction = Some(direction);
            transform.direction = direction;
        }
        Roll { direction } => {
            movement.motion = Motion::Rolling;
            movement.direction = Some(direction);
            transform.direction = direction;
        }
        Jump => {
            movement.motion = Motion::Jumping;
            movement.direction = None;
        }
    }
}

fn handle_movement(
    movement: &Movement,
    transform: &mut Transform,
) {
    if movement.motion == Motion::Idle {
        return;
    }

    //TODO: Use stat and base speed
    let mut speed = 1.0;

    //TODO: Use tick interval for delta time
    let velocity = speed * 0.1 * (*transform.direction);
    let velocity = Vector3::new(velocity.x, 0.0, velocity.y);
    transform.position += velocity;
}

//TODO: Add session as component?
pub fn sync(
    mut query: Query<(Entity, &mut Movement, &Transform), Changed<Movement>>,
    mut sessions: Query<(&SessionContext)>,
) {
    let mut states = Vec::new();
    query.iter_mut().for_each(|(entity, movement, transform)| {
        let state = MovementState {
            entity: entity.to_bits(),
            motion: movement.motion.into(),
            position: Some(transform.position.into()),
            direction: match movement.direction {
                Some(direction) => Some(direction.into()),
                None => None,
            },
        };
        states.push(state);
    });

    let protocol = MovementSync {
        timestamp: Timestamp::now().into(),
        states,
    };
    let buf = match protocol::encode(&protocol) {
        Ok(buf) => buf,
        Err(e) => {
            error!("Failed to encode: {}", e);
            return;
        }
    };

    sessions.iter().for_each(|session| {
        session.do_send(buf.clone());
    });
}

impl TryFrom<movement_command::Command> for MovementCommand {
    type Error = ();

    fn try_from(value: movement_command::Command) -> Result<Self, Self::Error> {
        use movement_command::Command;

        Ok(match value {
            Command::Halt(_) => MovementCommand::Halt,
            Command::Walk(walk) => {
                if walk.direction.is_none() {
                    return Err(());
                }

                MovementCommand::Walk { direction: walk.direction.unwrap().try_into()? }
            },
            Command::Run(run) => {
                if run.direction.is_none() {
                    return Err(());
                }

                MovementCommand::Run { direction: run.direction.unwrap().try_into()? }
            },
            Command::Roll(roll) => {
                if roll.direction.is_none() {
                    return Err(());
                }

                MovementCommand::Roll { direction: roll.direction.unwrap().try_into()? }
            },
        })
    }
}
