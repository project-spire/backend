use bevy_ecs::component::Component;
use nalgebra::{UnitVector2, Vector2};

use crate::calc::BasedValue;
use crate::physics::Speed;

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
