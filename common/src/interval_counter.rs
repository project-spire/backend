use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct IntervalCounter {
    window: VecDeque<Duration>,
    window_size: usize,

    duration_total: Duration,
    last_tick: Instant,
}

impl IntervalCounter {
    pub fn new(window_size: usize) -> Self {
        debug_assert!(window_size > 0);

        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            duration_total: Duration::from_secs(0),
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_tick);

        if self.window.len() == self.window_size {
            if let Some(duration) = self.window.pop_front() {
                self.duration_total = self.duration_total.saturating_sub(duration);
            }
        }

        self.window.push_back(duration);
        self.duration_total = self.duration_total.saturating_add(duration);

        self.last_tick = now;
    }

    /// Average seconds between ticks.
    pub fn average(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }

        if self.duration_total.as_secs() == 0 {
            return f64::INFINITY;
        }

        self.duration_total.as_secs_f64() / self.window.len() as f64
    }

    /// Average ticks in seconds.
    pub fn reversed(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }

        if self.duration_total.as_secs() == 0 {
            return f64::INFINITY;
        }

        self.window.len() as f64 / self.duration_total.as_secs_f64()
    }
}
