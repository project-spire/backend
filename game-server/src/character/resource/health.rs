use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
    
    pub state: State,
}

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Normal,
    Dying,
    Dead,
}
