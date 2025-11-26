use bevy_ecs::component::Component;

use crate::calc::BasedValue;
use crate::physics::Speed;

#[derive(Component)]
pub struct Movement {
    pub state: State,
    pub motion: Motion,
    
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
