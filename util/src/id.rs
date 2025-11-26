use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub type Id = i64;

const MAX_NODE_ID: u16 = (1 << 10) - 1;
const MAX_SEQUENCE: u16 = (1 << 12) - 1;
const CUSTOM_EPOCH: u64 = 1735689600000; // 2025-01-01 00:00:00 UTC

static GENERATOR: OnceLock<Generator> = OnceLock::new();

#[derive(Debug)]
pub struct Generator {
    node_id: u16,

    // High 42 bits: timestamp, Low 12 bits: sequence
    last_state: AtomicU64,
}

impl Generator {
    pub fn init(node_id: u16) {
        let generator = Self::new(node_id);
        GENERATOR.set(generator).expect("Generator already initialized");
    }

    fn new(node_id: u16) -> Self {
        if node_id > MAX_NODE_ID {
            panic!("Node ID {} exceeds maximum {}", node_id, MAX_NODE_ID);
        }

        Self { node_id, last_state: AtomicU64::new(0) }
    }

    /// Generate a 64-bit universally unique id.
    fn generate(&self) -> Id {
        loop {
            let timestamp = current_timestamp();

            let last_state = self.last_state.load(Ordering::Acquire);
            let last_timestamp = last_state >> 12;
            let last_sequence = (last_state & 0xFFF) as u16;

            let (new_timestamp, sequence) = if timestamp == last_timestamp {
                let next_sequence = last_sequence + 1;

                if next_sequence >= MAX_SEQUENCE {
                    // Sequence overflow, wait for next millisecond
                    std::hint::spin_loop();
                    continue;
                }

                (timestamp, next_sequence)
            } else if timestamp > last_timestamp {
                (timestamp, 0)
            } else {
                panic!("Clock moved backwards!");
            };

            let new_state = (new_timestamp << 12) | (sequence as u64);
            match self.last_state.compare_exchange_weak(
                last_state,
                new_state,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let id = ((new_timestamp << 22) |
                        ((self.node_id as u64) << 12) |
                        (sequence as u64)) as i64;

                    return id;
                }
                Err(_) => {
                    // Contention, retry
                    std::hint::spin_loop();
                    continue;
                }
            }
        }
    }
}

pub fn generate() -> i64 {
    GENERATOR
        .get()
        .expect("Generator not initialized yet!")
        .generate()
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        - CUSTOM_EPOCH
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_unique_ids() {
        let generator = Arc::new(Generator::new(1));
        let mut handles = vec![];
        let mut all_ids = HashSet::new();

        // Generate IDs from multiple threads
        for _ in 0..10 {
            let generator = Arc::clone(&generator);
            let handle = thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..1000 {
                    ids.push(generator.generate());
                }
                ids
            });
            handles.push(handle);
        }

        // Collect all IDs
        for handle in handles {
            let ids = handle.join().unwrap();
            for id in ids {
                assert!(all_ids.insert(id), "Duplicate ID found: {}", id);
            }
        }

        println!("Generated {} unique IDs", all_ids.len());
    }

    #[test]
    fn test_id_components() {
        let generator = Generator::new(123);
        let id = generator.generate();

        // Extract components
        let sequence = id & 0xFFF;
        let node_id = (id >> 12) & 0x3FF;
        let timestamp = (id >> 22) & 0x1FFFFFFFFFF;

        assert_eq!(node_id, 123);
        assert!(sequence < MAX_SEQUENCE as i64);
        assert!(timestamp > 0);

        println!("ID: {}", id);
        println!("Timestamp: {}", timestamp);
        println!("Node ID: {}", node_id);
        println!("Sequence: {}", sequence);
    }
}
