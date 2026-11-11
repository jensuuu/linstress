//! RAM stress test: allocates memory, writes known patterns, reads them back
//! and counts every value that changed. Errors here usually mean unstable
//! memory (for example a bad RAM overclock or XMP/EXPO profile).

use std::ptr::{read_volatile, write_volatile};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Number of different patterns used per cycle.
const PATTERN_COUNT: usize = 5;
/// How many words to process before checking the stop flag.
const CHECK_INTERVAL: usize = 1 << 20;

/// Counters shared between the worker threads and the UI.
#[derive(Default)]
struct RamStats {
    /// Finished write + verify rounds, summed over all threads.
    cycles: AtomicU64,
    /// Values that did not match what was written.
    errors: AtomicU64,
    /// True when memory could not be allocated.
    allocation_failed: AtomicBool,
}

pub struct RamStress {
    stop_flag: Arc<AtomicBool>,
    stats: Arc<RamStats>,
    workers: Vec<JoinHandle<()>>,
}

impl RamStress {
    /// Tests `size_mb` megabytes of RAM, split evenly over `thread_count` threads.
    pub fn start(size_mb: usize, thread_count: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(RamStats::default());
        let words_per_thread = size_mb * 1024 * 1024 / 8 / thread_count.max(1);

        let workers = (0..thread_count.max(1))
            .map(|_| {
                let stop = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                thread::spawn(move || run_memory_test(words_per_thread, &stop, &stats))
            })
            .collect();

        Self { stop_flag, stats, workers }
    }

    pub fn cycles(&self) -> u64 {
        self.stats.cycles.load(Ordering::Relaxed)
    }

    pub fn errors(&self) -> u64 {
        self.stats.errors.load(Ordering::Relaxed)
    }

    pub fn allocation_failed(&self) -> bool {
        self.stats.allocation_failed.load(Ordering::Relaxed)
    }

    /// Signals all workers to stop, waits for them, and frees the memory.
    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        for worker in self.workers {
            let _ = worker.join();
        }
    }
}

/// The value that should be stored at `index` during pattern round `round`.
fn expected_value(round: usize, index: usize) -> u64 {
    match round % PATTERN_COUNT {
        0 => 0x0000_0000_0000_0000,
        1 => 0xFFFF_FFFF_FFFF_FFFF,
        2 => 0xAAAA_AAAA_AAAA_AAAA,
        3 => 0x5555_5555_5555_5555,
        // Every address gets a different value, which also catches addressing faults.
        _ => (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
    }
}

fn run_memory_test(words: usize, stop: &AtomicBool, stats: &RamStats) {
    // Ask for the memory politely instead of crashing when it is not available.
    let mut buffer: Vec<u64> = Vec::new();
    if buffer.try_reserve_exact(words).is_err() {
        stats.allocation_failed.store(true, Ordering::Relaxed);
        return;
    }
    buffer.resize(words, 0);

    let mut round = 0;
    while !stop.load(Ordering::Relaxed) {
        // Write the pattern. Volatile access stops the compiler from skipping the work.
        for (index, word) in buffer.iter_mut().enumerate() {
            if index % CHECK_INTERVAL == 0 && stop.load(Ordering::Relaxed) {
                return;
            }
            // SAFETY: `word` is a valid, aligned reference into our own buffer.
            unsafe { write_volatile(word, expected_value(round, index)) };
        }

        // Read everything back and count the differences.
        let mut errors = 0;
        for (index, word) in buffer.iter().enumerate() {
            if index % CHECK_INTERVAL == 0 && stop.load(Ordering::Relaxed) {
                return;
            }
            // SAFETY: same as above.
            let stored = unsafe { read_volatile(word) };
            if stored != expected_value(round, index) {
                errors += 1;
            }
        }

        stats.errors.fetch_add(errors, Ordering::Relaxed);
        stats.cycles.fetch_add(1, Ordering::Relaxed);
        round += 1;
    }
}
