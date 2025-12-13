use std::ops::Deref;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct BasedValue<T>
where
    T: Default,
{
    pub base: T,
    pub current: T,
}

pub struct Ticker {
    last_tick: Instant,
    duration: Duration,
}

impl<T> BasedValue<T>
where
    T: Copy + Default,
{
    pub fn new(base: T) -> Self {
        Self {
            base,
            current: base,
        }
    }

    pub fn reset(&mut self) {
        self.current = self.base;
    }
}

impl<T> Deref for BasedValue<T>
where
    T: Default,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl Ticker {
    pub fn new(duration: Duration) -> Self {
        Self {
            last_tick: Instant::now(),
            duration,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.last_tick.elapsed() > self.duration {
            self.last_tick += self.duration;
            true
        } else {
            false
        }
    }
}
