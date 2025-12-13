use crate::calc::{BasedValue, Ticker};
use bevy_ecs::prelude::*;
use nalgebra::clamp;

#[derive(Component)]
pub struct Health {
    pub state: State,

    pub current: u64,
    pub max: BasedValue<u64>,
    pub regen: BasedValue<u64>,
    regen_ticker: Ticker,
}

#[derive(Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Alive,
    Dying,
    Dead,
}

pub fn regenerate(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        if health.state != State::Alive {
            continue;
        }
        
        if !health.regen_ticker.tick() {
            continue;
        }

        if *health.regen == 0 {
            continue;
        }

        health.current = clamp(
            health.current.saturating_add(*health.regen),
            0,
            *health.max,
        );
    }
}
