use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct WorldTime {
    pub now: std::time::Instant,
    pub dt: std::time::Duration,
}

impl Default for WorldTime {
    fn default() -> Self {
        WorldTime { now: std::time::Instant::now(), dt: std::time::Duration::default() }
    }
}