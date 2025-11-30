use std::fmt;
use std::fmt::Formatter;
use std::time::Instant;

use serde::Deserialize;

pub struct RateLimiter {
    rate: f32,
    capacity: f32,

    current: f32,
    last_update: Instant,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Params {
    rate: f32,
    capacity: f32,
}

#[derive(Debug)]
pub enum Error {
    Exceed { rate: f32, capacity: f32 },
}

impl RateLimiter {
    pub fn new(params: Params) -> Self {
        Self {
            rate: params.rate,
            capacity: params.capacity,
            current: params.capacity,
            last_update: Instant::now(),
        }
    }

    pub fn check(&mut self) -> Result<(), Error> {
        self.check_internal(1.0)
    }

    pub fn check_with_value(&mut self, value: f32) -> Result<(), Error> {
        self.check_internal(value)
    }

    fn check_internal(&mut self, value: f32) -> Result<(), Error> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f32();

        // Refill tokens.
        self.current += elapsed * self.rate;
        if self.current > self.capacity {
            self.current = self.capacity;
        }

        self.last_update = now;

        // Consume token.
        if self.current >= value {
            self.current -= value;
            Ok(())
        } else {
            Err(Error::Exceed { rate: self.rate, capacity: self.capacity })
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Error::Exceed { rate, capacity } => write!(
                f,
                "Rate exceeded the limit: rate={}, capacity={}",
                rate,
                capacity,
            ),
        }
    }
}

impl std::error::Error for Error {}
