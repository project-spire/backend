use bevy_ecs::prelude::*;
use crate::character::stat::MobilityStat;
use crate::character::status_effect::StatusEffectController;
use crate::physics::object::Transform;
use crate::protocol::*;
use crate::protocol::game::*;
use crate::world::time::WorldTime;
use nalgebra::{Point2, UnitVector2, Vector2};

use crate::character::movement::MovementState::*;
use crate::character::movement::MovementMode::*;
use crate::character::movement::MovementInterpolation::*;
use crate::character::movement::MovementCommand::*;

type Transition = (MovementState, std::time::Instant);

#[derive(Eq, PartialEq, Copy, Clone, Default)]
pub enum MovementState {
    #[default]
    Idle = 0,
    Walking = 1,
    Running = 2,
    Rolling = 3,
}

#[derive(Eq, PartialEq, Copy, Clone, Default)]
pub enum MovementMode {
    #[default]
    Standing = 0,
    Crouching = 1,
    Crawling = 2,
    Swimming = 3,
    Flying = 4,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum MovementInterpolation {
    None = 0,
    Linear = 1,
    DeadReckoning = 2,
}

pub enum MovementCommand {
    // State changes
    Halt,
    Walk { direction: UnitVector2<f32> },
    Run { direction: UnitVector2<f32> },
    Roll { direction: UnitVector2<f32> },

    // Mode changes
    Stand,
    Crouch,
    Crawl,
    Swim,
    Fly,

    // etc
    Teleport { position: Point2<f32>, forced: bool },
}

#[derive(Component, Default)]
pub struct MovementController {
    state: MovementState,
    mode: MovementMode,
    commands: Vec<MovementCommand>,
    transition: Option<Transition>,
    interpolation: Option<MovementInterpolation>,
}

pub fn update(
    mut query: Query<(
        &mut MovementController,
        &mut Transform,
        &MobilityStat,
        Option<&StatusEffectController>)>,
    time: Res<WorldTime>,
) {
    query.iter_mut().for_each(
        |(mut controller, mut transform, mobility, status)| {
        if let Some(transition) = controller.transition.take() {
            handle_transition(transition, &mut controller, &time);
        }

        let commands: Vec<_> = controller.commands.drain(..).collect();
        for command in commands {
            handle_command(command, &mut controller, &mut transform, status);
        }

        handle_movement(&controller, &mut transform, &mobility, &time);
    })
}

fn handle_transition(
    transition: Transition,
    controller: &mut MovementController,
    time: &Res<WorldTime>,
) {
    let (state, then) = transition;

    if time.now < then {
        controller.transition = Some((state, then)); // give back
        return;
    }

    controller.state = state;
}

fn handle_command(
    command: MovementCommand,
    controller: &mut MovementController,
    transform: &mut Transform,
    status: Option<&StatusEffectController>,
) {
    match command {
        Halt => if controller.state == Walking || controller.state == Running {
            controller.state = Idle;
        }
        Walk { direction } => {
            //TODO: Check status
            controller.state = Walking;
            transform.rotation = direction;
        }
        Run { direction } => {
            //TODO: Check status
            controller.state = Running;
            transform.rotation = direction;
        }
        Roll { direction } => {
            //TODO: Check status
            //TODO: Set rolling expiration as transition
            controller.state = Rolling;
            transform.rotation = direction;
        }
        Stand => {
            //TODO: Check status
            controller.mode = Standing;
        }
        Crouch => {
            //TODO: Check status
            controller.mode = Crouching;
        }
        Crawl => {
            //TODO: Check status
            controller.mode = Crawling;
        }
        Swim => {
            //TODO: Check status
            controller.mode = Swimming;
        }
        Fly => {
            //TODO: Check status
            controller.mode = Flying;
        }
        Teleport { position, forced } => {
            controller.interpolation = Some(None); // Don't interpolate the teleportation

            if forced {
                //TODO: Check if movable to the position
                transform.position = position;
                return;
            }

            //TODO: Check status
            transform.position = position;
        }
    }
}

fn handle_movement(
    controller: &MovementController,
    transform: &mut Transform,
    mobility: &MobilityStat,
    time: &Res<WorldTime>,
) {
    //TODO: Extract to config?
    const WALK_MULTIPLIER: f32 = 1.0;
    const RUN_MULTIPLIER: f32 = 1.5;

    const STAND_MULTIPLIER: f32 = 1.0;
    const CROUCH_MULTIPLIER: f32 = 0.6;
    const CRAWL_MULTIPLIER: f32 = 0.3;

    if controller.state == Idle {
        transform.velocity = Vector2::<f32>::zeros();
        return;
    }

    if controller.state == Rolling {
        //TODO
        return;
    }

    let mut speed = mobility.speed;
    speed *= match controller.state {
        Walking => WALK_MULTIPLIER,
        Running => RUN_MULTIPLIER,
        _ => 1.0,
    };
    speed *= match controller.mode {
        Standing => STAND_MULTIPLIER,
        Crouching => CROUCH_MULTIPLIER,
        Crawling => CRAWL_MULTIPLIER,
        _ => 1.0,
    };

    let velocity = speed * (time.dt.as_millis() as f32) * (*transform.rotation);
    transform.position += velocity;
    transform.velocity = velocity;
}

//TODO: Add session as component?
pub fn sync(
    mut query: Query<(Entity, &mut MovementController, &Transform), Changed<MovementController>>,
) {
    //TODO: initialize Vec with query size
    let mut movements = Vec::new();

    query.iter_mut().for_each(|(entity, mut controller, transform)| {
        let interpolation = if let Some(interpolation) = controller.interpolation.take() {
            interpolation
        } else {
            // Default interpolation is linear
            Linear
        };

        let movement = Movement {
            entity: entity.to_bits(),
            state: controller.state as i32,
            mode: controller.mode as i32,
            interpolation: interpolation as i32,
            position: Some(transform.position.into()),
            velocity: Some(transform.velocity.into()),
        };
        movements.push(movement);
    });

    let protocol = GameServerProtocol {
        protocol: Some(game_server_protocol::Protocol::MovementSync(MovementSync { movements }))
    };
    let buf = serialize_protocol(ProtocolCategory::Game, &protocol);
    if let Err(e) = buf {
        //TODO: Log
        return;
    }

    //TODO: How to broadcast?
}
