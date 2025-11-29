use bevy_ecs::prelude::*;
use nalgebra::{UnitVector2, Vector2};

use crate::calc::BasedValue;
use crate::physics::Speed;
use crate::world::time::Time;
use crate::world::transform::Transform;

#[derive(Component)]
pub struct Movement {
    pub state: State,
    pub motion: Motion,

    pub direction: UnitVector2<f32>,
    pub speed: BasedValue<Speed>,
}

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Normal,
    Bound,
}

#[derive(Debug, Default)]
pub enum Motion {
    #[default]
    Idle,
    Walking,
    Running,
    Jumping,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            state: State::default(),
            motion: Motion::default(),
            direction: UnitVector2::new_normalize(Vector2::new(1.0, 0.0)),
            speed: BasedValue::new(Speed::default()),
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(move_position);
}

fn move_position(
    mut query: Query<(&Movement, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (movement, mut transform) in query.iter_mut() {
        if let Motion::Idle = movement.motion {
            continue;
        }

        // TODO: Handle motions case

        let distance = *movement.speed * dt;
        let dx = movement.direction.x * distance;
        let dz = movement.direction.y * distance;

        // TODO: Check if moveable to the target position
        transform.position.x += dx;
        transform.position.z += dz;
    }
}
