//! CPU stress test: spawns worker threads that run heavy floating-point math
//! until they are told to stop.

use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct StressTest {
    stop_flag: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}

impl StressTest {
    /// Starts `thread_count` busy worker threads.
    pub fn start(thread_count: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let workers = (0..thread_count)
            .map(|_| {
                let stop = Arc::clone(&stop_flag);
                thread::spawn(move || burn_cpu(&stop))
            })
            .collect();

        Self { stop_flag, workers }
    }

    /// Signals all workers to stop and waits for them to finish.
    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        for worker in self.workers {
            let _ = worker.join();
        }
    }
}

/// Heavy math loop. `black_box` stops the compiler from optimizing the work away.
fn burn_cpu(stop: &AtomicBool) {
    let mut value = 1.000_1_f64;

    while !stop.load(Ordering::Relaxed) {
        // Check the stop flag only every million iterations to keep the CPU busy.
        for _ in 0..1_000_000 {
            value = black_box(value.sin().cos().sqrt() + value.ln_1p() * 1.000_001);
            if !value.is_finite() || value > 1e6 {
                value = 1.000_1;
            }
        }
    }
}
