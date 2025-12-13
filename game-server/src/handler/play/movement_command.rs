use bevy_ecs::prelude::*;

use crate::character::status::movement::{Motion, Movement};
use crate::handler::ProtocolLocalHandler;
use protocol::game::play::movement_command::{*, Command};
use protocol::game::play::MovementCommand;

impl ProtocolLocalHandler for MovementCommand {
    fn handle(self, world: &mut World, entity: Entity) {
        let Some(command) = self.command else {
            return;
        };

        let Some(mut movement) = world.get_mut::<Movement>(entity) else {
            return;
        };

        match command {
            Command::Halt(halt) => handle_halt(halt, &mut movement),
            Command::Walk(walk) => handle_walk(walk, &mut movement),
            Command::Run(run) => handle_run(run, &mut movement),
            Command::Roll(roll) => handle_roll(roll, &mut movement),
            Command::Jump(jump) => handle_jump(jump, &mut movement),
        }
    }
}

fn handle_halt(_: Halt, movement: &mut Mut<Movement>) {
    if movement.motion == Motion::Idle {
        return;
    }

    movement.motion = Motion::Idle;
}

fn handle_walk(walk: Walk, movement: &mut Mut<Movement>) {
    if !movement.can_move() {
        return;
    }

    let Some(direction) = walk.direction.and_then(|d| d.try_into().ok()) else {
        return;
    };

    movement.motion = Motion::Walking;
    movement.direction = direction;
}

fn handle_run(run: Run, movement: &mut Mut<Movement>) {
    if !movement.can_move() {
        return;
    }

    let Some(direction) = run.direction.and_then(|d| d.try_into().ok()) else {
        return;
    };

    movement.motion = Motion::Running;
    movement.direction = direction;
}

fn handle_roll(roll: Roll, movement: &mut Mut<Movement>) {
    if !movement.can_move() {
        return;
    }

    let Some(direction) = roll.direction.and_then(|d| d.try_into().ok()) else {
        return;
    };

    movement.motion = Motion::Rolling;
    movement.direction = direction;
}

fn handle_jump(_: Jump, movement: &mut Mut<Movement>) {
    if !movement.can_jump() {
        return;
    }

    movement.motion = Motion::Jumping;
}
