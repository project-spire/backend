use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};


pub type Id = i64;


const CUSTOM_EPOCH: u64 = 1735689600000; // 2025-01-01 00:00:00 UTC;
const UNIVERSAL_NODE_BITS: u8 = 10;
const UNIVERSAL_SEQUENCE_BITS: u8 = 12;
const GLOBAL_SEQUENCE_BITS: u8 = 22;


static UNIVERSAL_NODE_ID: OnceLock<u16> = OnceLock::new();

/// High 10 bits: empty, Middle 42 bits: timestamp, Low 12 bits: sequence
static UNIVERSAL_LAST_STATE: AtomicU64 = AtomicU64::new(0);

/// High 42 bits: timestamp, Low 22 bits: sequence
static GLOBAL_LAST_STATE: AtomicU64 = AtomicU64::new(0);


/// Initialize the id generator.
pub fn init(node_id: u16) {
    let node_id_max = (1 << UNIVERSAL_NODE_BITS) - 1;
    if node_id > node_id_max {
        panic!("Node ID {} exceeds maximum {}", node_id, node_id_max);
    }

    UNIVERSAL_NODE_ID.set(node_id).expect("Universal node ID already initialized");
}

/// Generate a 64-bit universally unique id.
pub fn universal() -> Id {
    let node_id = *UNIVERSAL_NODE_ID.get().expect("Universal node ID not initialized");
    let (timestamp, sequence) = acquire_new_state(&UNIVERSAL_LAST_STATE, UNIVERSAL_SEQUENCE_BITS);

    ((timestamp << (UNIVERSAL_NODE_BITS + UNIVERSAL_SEQUENCE_BITS)) |
        ((node_id as u64) << UNIVERSAL_SEQUENCE_BITS) |
        (sequence as u64)) as i64
}

/// Decompose a universally unique id into timestamp, node id, and sequence.
pub fn universal_decompose(id: Id) -> (u64, u16, u16) {
    let timestamp = (id >> (UNIVERSAL_NODE_BITS + UNIVERSAL_SEQUENCE_BITS)) as u64 + CUSTOM_EPOCH;
    let node_id = ((id >> UNIVERSAL_SEQUENCE_BITS) & ((1 << UNIVERSAL_NODE_BITS) - 1)) as u16;
    let sequence = (id & ((1 << UNIVERSAL_SEQUENCE_BITS) - 1)) as u16;

    (timestamp, node_id, sequence)
}

/// Generate a 64-bit globally unique id.
pub fn global() -> Id {
    let (timestamp, sequence) = acquire_new_state(&GLOBAL_LAST_STATE, GLOBAL_SEQUENCE_BITS);

    ((timestamp << GLOBAL_SEQUENCE_BITS) | (sequence as u64)) as i64
}

/// Decompose a globally unique id into timestamp and sequence.
pub fn global_decompose(id: Id) -> (u64, u32) {
    let timestamp = (id >> GLOBAL_SEQUENCE_BITS) as u64 + CUSTOM_EPOCH;
    let sequence = (id & ((1 << GLOBAL_SEQUENCE_BITS) - 1)) as u32;

    (timestamp, sequence)
}

fn acquire_new_state(
    last_state_target: &AtomicU64,
    sequence_bits: u8,
) -> (u64, u32) {
    let sequence_max = (1 << sequence_bits) - 1;

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_millis() as u64
            - CUSTOM_EPOCH;

        let last_state = last_state_target.load(Ordering::Acquire);
        let last_timestamp = last_state >> sequence_bits;
        let last_sequence = (last_state & sequence_max as u64) as u32;

        let (new_timestamp, sequence) = if timestamp == last_timestamp {
            let next_sequence = last_sequence + 1;

            if next_sequence >= sequence_max {
                // Sequence overflow, wait for the next millisecond.
                std::hint::spin_loop();
                continue;
            }

            (timestamp, next_sequence)
        } else if timestamp > last_timestamp {
            (timestamp, 0)
        } else {
            // Clock moved backwards, wait.
            std::thread::yield_now();
            continue;
        };

        let new_state = (new_timestamp << sequence_bits) | (sequence as u64);
        if last_state_target.compare_exchange_weak(
            last_state,
            new_state,
            Ordering::Release,
            Ordering::Acquire,
        ).is_ok() {
            return (new_timestamp, sequence);
        }

        // Contention, retry.
        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;

    const TEST_NODE_ID: u16 = 777;

    fn test_init() {
        _ = UNIVERSAL_NODE_ID.set(TEST_NODE_ID);
    }

    fn generate_id_concurrent<F>(
        thread_count: usize,
        id_count: usize,
        generate: F,
    ) -> HashSet<Id>
    where
        F: Fn() -> Id,
        F: Copy + Send + 'static,
    {
        let mut handles = vec![];
        let mut all_ids = HashSet::new();

        // Generate IDs from multiple threads
        for _ in 0..thread_count {
            handles.push(thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..id_count {
                    ids.push(generate());
                }
                ids
            }));
        }

        // Collect all IDs
        for handle in handles {
            let ids = handle.join().unwrap();
            for id in ids {
                assert!(all_ids.insert(id), "Duplicate ID found: {}", id);
            }
        }

        all_ids
    }

    #[test]
    fn test_universal_id_uniqueness() {
        test_init();

        let thread_count = 10;
        let id_count = 4096;

        let all_ids = generate_id_concurrent(
            thread_count,
            id_count,
            || { universal() },
        );
        assert_eq!(all_ids.len(), thread_count * id_count);
    }

    #[test]
    fn test_global_id_uniqueness() {
        test_init();

        let thread_count = 10;
        let id_count = 4096;

        let all_ids = generate_id_concurrent(
            thread_count,
            id_count,
            || { global() },
        );
        assert_eq!(all_ids.len(), thread_count * id_count);
    }

    #[test]
    fn test_universal_id_components() {
        test_init();

        let id = universal();
        let (timestamp, node_id, sequence) = universal_decompose(id);

        assert!(timestamp >= CUSTOM_EPOCH);
        assert_eq!(node_id, TEST_NODE_ID);
        assert!(sequence < (1 << UNIVERSAL_SEQUENCE_BITS));
    }

    #[test]
    fn test_global_id_components() {
        test_init();

        let id = global();
        let (timestamp, sequence) = global_decompose(id);

        assert!(timestamp >= CUSTOM_EPOCH);
        assert!(sequence < (1 << GLOBAL_SEQUENCE_BITS));
    }

    #[test]
    fn test_universal_id_monotonicity() {
        test_init();

        let mut previous_id = universal();
        for _ in 0..1000 {
            let id = universal();
            assert!(id > previous_id, "Universal IDs must be strictly increasing");
            previous_id = id;
        }
    }

    #[test]
    fn test_global_id_monotonicity() {
        test_init();

        let mut previous_id = global();
        for _ in 0..1000 {
            let id = global();
            assert!(id > previous_id, "Local IDs must be strictly increasing");
            previous_id = id;
        }
    }
}
