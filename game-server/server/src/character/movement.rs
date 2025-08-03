use bevy_ecs::prelude::*;
use nalgebra::{UnitVector2, Vector2, Vector3};
use tracing::error;
use crate::network::session::{SessionContext};
use crate::protocol::{*, game::*};
use crate::timestamp::Timestamp;
use crate::world::transform::Transform;

pub enum MovementCommand {
    Halt,
    Walk { direction: UnitVector2<f32> },
    Run { direction: UnitVector2<f32> },
    Roll { direction: UnitVector2<f32> },
    Jump,
}

impl From<movement_command_protocol::Command> for MovementCommand {
    fn from(value: movement_command_protocol::Command) -> Self {
        use movement_command_protocol::Command;

        match value {
            Command::Halt(_) => MovementCommand::Halt,
            Command::Walk(walk) => MovementCommand::Walk { direction: walk.direction.unwrap().into() },
            Command::Run(run) => MovementCommand::Run { direction: run.direction.unwrap().into() },
            Command::Roll(roll) => MovementCommand::Roll { direction: roll.direction.unwrap().into() },
        }
    }
}

// impl From<HaltMovementCommand> for MovementCommand {
//     fn from(_: HaltMovementCommand) -> Self {
//         MovementCommand::Halt
//     }
// }
//
// impl From<WalkMovementCommand> for MovementCommand {
//     fn from(value: WalkMovementCommand) -> Self {
//         MovementCommand::Walk {
//             direction: value.direction.into(),
//         }
//     }
// }
//
// impl From<RunMovementCommand> for MovementCommand {
//     fn from(value: RunMovementCommand) -> Self {
//         MovementCommand::Run {
//             direction: value.direction.into(),
//         }
//     }
// }
//
// impl From<RollMovementCommand> for MovementCommand {
//     fn from(value: RollMovementCommand) -> Self {
//         MovementCommand::Roll {
//             direction: value.direction.into(),
//         }
//     }
// }
//
// impl From<JumpMovementCommand> for MovementCommand {
//     fn from(_: JumpMovementCommand) -> Self {
//         MovementCommand::Jump
//     }
// }

#[derive(Component, Default)]
pub struct Movement {
    state: MovementState,
    direction: Option<UnitVector2<f32>>,
    commands: Vec<(Timestamp, MovementCommand)>,
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
            movement.state = MovementState::Idle;
            movement.direction = None;
        },
        Walk { direction } => {
            movement.state = MovementState::Walking;
            movement.direction = Some(direction);
            transform.direction = direction;
        }
        Run { direction } => {
            movement.state = MovementState::Running;
            movement.direction = Some(direction);
            transform.direction = direction;
        }
        Roll { direction } => {
            movement.state = MovementState::Rolling;
            movement.direction = Some(direction);
            transform.direction = direction;
        }
        Jump => {
            movement.state = MovementState::Jumping;
            movement.direction = None;
        }
    }
}

fn handle_movement(
    movement: &Movement,
    transform: &mut Transform,
) {
    if movement.state == MovementState::Idle {
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
    //TODO: initialize Vec with query size
    let mut syncs = Vec::new();

    query.iter_mut().for_each(|(entity, movement, transform)| {
        let direction: PVector2 = match movement.direction {
            Some(d) => d.into(),
            None => PVector2 { x: 0.0, y: 0.0 },
        };

        let sync = MovementSync {
            entity: entity.to_bits(),
            state: movement.state as i32,
            position: Some(transform.position.into()),
            direction: Some(direction),
        };
        syncs.push(sync);
    });

    let buf = match encode_game(&GameServerProtocol {
        protocol: Some(game_server_protocol::Protocol::MovementSyncs(MovementSyncProtocol { syncs }))
    }) {
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
