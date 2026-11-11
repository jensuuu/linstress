//! The application state and the logic behind the buttons.
//! The look of each part of the window lives in `components/`.

use std::time::Duration;

use gpui::{div, prelude::*, rgb, Context, Window};

use crate::cpu_stress::CpuStress;
use crate::gpu_stress::GpuStress;
use crate::ram_stress::RamStress;
use crate::overclock::NvidiaInfo;
use crate::sensors::{self, CpuUsageMeter};
use crate::{overclock, theme};

/// The RAM test never uses more than this share of the free memory (percent).
const MAX_RAM_SHARE_OF_FREE: u64 = 90;
/// How much the RAM size changes per click.
pub const RAM_STEP_MB: i32 = 256;
/// Number of threads used by the RAM test (at most).
const RAM_THREADS: usize = 8;
/// Lowest core clock the NVIDIA clock lock may use (MHz) and its step size.
const NVIDIA_CLOCK_FLOOR_MHZ: u32 = 210;
pub const NVIDIA_CLOCK_STEP_MHZ: i32 = 30;
pub const NVIDIA_POWER_STEP_W: i32 = 5;

pub struct StressApp {
    // Live readings
    pub cpu_usage: f32,
    pub cpu_frequency_mhz: Option<f32>,
    pub cpu_temperature: Option<f32>,
    pub gpu_temperature: Option<f32>,
    pub memory: Option<(u64, u64)>, // (total MB, available MB)

    // The three stress tests (None = not running)
    pub cpu_test: Option<CpuStress>,
    pub ram_test: Option<RamStress>,
    pub gpu_test: Option<GpuStress>,

    // Test settings
    pub cpu_threads: usize,
    pub ram_size_mb: usize,
    pub temperature_limit: f32, // All tests stop automatically above this.

    // Tuning
    pub target_max_mhz: u32,
    pub frequency_range: Option<(u32, u32)>, // Hardware minimum and maximum.
    pub nvidia: Option<NvidiaInfo>,          // None = no NVIDIA GPU found.
    pub nvidia_power_w: u32,
    pub nvidia_clock_mhz: u32,

    // Helpers
    usage_meter: CpuUsageMeter,
    pub log: Vec<String>,
}

impl StressApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Refresh the sensor values once per second.
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            let window_is_open = this
                .update(cx, |this, cx| {
                    this.refresh_sensors();
                    cx.notify(); // Ask GPUI to redraw.
                })
                .is_ok();
            if !window_is_open {
                break;
            }
        })
        .detach();

        let memory = sensors::memory_mb();
        let default_ram_mb = memory
            .map(|(_, available)| (available / 4).clamp(256, 2048) as usize)
            .unwrap_or(1024);

        let nvidia = overclock::nvidia_info();

        Self {
            cpu_usage: 0.0,
            cpu_frequency_mhz: None,
            cpu_temperature: None,
            gpu_temperature: None,
            memory,
            cpu_test: None,
            ram_test: None,
            gpu_test: None,
            cpu_threads: sensors::logical_cpu_count(),
            ram_size_mb: default_ram_mb,
            temperature_limit: 90.0,
            target_max_mhz: overclock::current_max_mhz().unwrap_or(3000),
            frequency_range: overclock::cpu_frequency_limits_mhz(),
            nvidia,
            nvidia_power_w: nvidia.map(|info| info.power_now_w).unwrap_or(200),
            nvidia_clock_mhz: nvidia.map(|info| info.clock_max_mhz).unwrap_or(1500),
            usage_meter: CpuUsageMeter::new(),
            log: vec!["Ready. Tuning actions need root permission.".into()],
        }
    }

    // --- Sensors -----------------------------------------------------------

    fn refresh_sensors(&mut self) {
        self.cpu_usage = self.usage_meter.sample();
        self.cpu_frequency_mhz = sensors::cpu_frequency_mhz();
        self.cpu_temperature = sensors::cpu_temperature_celsius();
        self.gpu_temperature = sensors::gpu_temperature_celsius();
        self.memory = sensors::memory_mb();

        // Safety: stop every test automatically if it gets too hot.
        let hottest = self
            .cpu_temperature
            .unwrap_or(0.0)
            .max(self.gpu_temperature.unwrap_or(0.0));

        if self.any_running() && hottest >= self.temperature_limit {
            self.stop_all();
            self.add_log(format!("Safety stop: {hottest:.0} °C reached the limit"));
        }
    }

    pub fn has_temperature_sensor(&self) -> bool {
        self.cpu_temperature.is_some() || self.gpu_temperature.is_some()
    }

    // --- Log ---------------------------------------------------------------

    pub fn add_log(&mut self, message: String) {
        self.log.push(message);
        if self.log.len() > 8 {
            self.log.remove(0); // Keep only the latest lines.
        }
    }

    pub fn log_result(&mut self, result: Result<String, String>) {
        match result {
            Ok(message) => self.add_log(message),
            Err(error) => self.add_log(format!("Error: {error}")),
        }
    }

    // --- Starting and stopping tests --------------------------------------

    pub fn any_running(&self) -> bool {
        self.cpu_test.is_some() || self.ram_test.is_some() || self.gpu_test.is_some()
    }

    pub fn toggle_cpu_test(&mut self) {
        if let Some(test) = self.cpu_test.take() {
            test.stop();
            self.add_log("CPU stress test stopped".into());
        } else {
            self.cpu_test = Some(CpuStress::start(self.cpu_threads));
            self.add_log(format!("CPU stress test started on {} threads", self.cpu_threads));
        }
    }

    pub fn toggle_ram_test(&mut self) {
        if let Some(test) = self.ram_test.take() {
            let (cycles, errors) = (test.cycles(), test.errors());
            test.stop();
            self.add_log(format!("RAM stress test stopped ({cycles} cycles, {errors} errors)"));
            return;
        }

        // Refuse to use more memory than is free, so the system does not freeze.
        let available_mb = self.memory.map(|(_, available)| available).unwrap_or(0);
        let allowed_mb = available_mb * MAX_RAM_SHARE_OF_FREE / 100;
        if self.ram_size_mb as u64 > allowed_mb {
            self.add_log(format!(
                "Not enough free RAM: {available_mb} MB free, {} MB requested",
                self.ram_size_mb
            ));
            return;
        }

        let threads = sensors::logical_cpu_count().min(RAM_THREADS);
        self.ram_test = Some(RamStress::start(self.ram_size_mb, threads));
        self.add_log(format!("RAM stress test started on {} MB", self.ram_size_mb));
    }

    /// Setting up the GPU takes a moment, so the window may pause briefly.
    pub fn toggle_gpu_test(&mut self) {
        if let Some(test) = self.gpu_test.take() {
            test.stop();
            self.add_log("GPU stress test stopped".into());
            return;
        }

        match GpuStress::start() {
            Ok(test) => {
                self.add_log(format!("GPU stress test started on {}", test.adapter_name()));
                self.gpu_test = Some(test);
            }
            Err(message) => self.add_log(format!("GPU test failed: {message}")),
        }
    }

    pub fn stop_all(&mut self) {
        if let Some(test) = self.cpu_test.take() {
            test.stop();
        }
        if let Some(test) = self.ram_test.take() {
            test.stop();
        }
        if let Some(test) = self.gpu_test.take() {
            test.stop();
        }
        self.add_log("All stress tests stopped".into());
    }

    // --- Settings ----------------------------------------------------------

    pub fn change_cpu_threads(&mut self, delta: i32) {
        let max = sensors::logical_cpu_count() as i32;
        self.cpu_threads = (self.cpu_threads as i32 + delta).clamp(1, max) as usize;
    }

    pub fn change_ram_size(&mut self, delta: i32) {
        let max_mb = self.memory.map(|(total, _)| total as i32).unwrap_or(65536);
        self.ram_size_mb = (self.ram_size_mb as i32 + delta).clamp(RAM_STEP_MB, max_mb) as usize;
    }

    pub fn change_temperature_limit(&mut self, delta: f32) {
        self.temperature_limit = (self.temperature_limit + delta).clamp(60.0, 105.0);
    }

    pub fn change_target_mhz(&mut self, delta: i32) {
        let (min, max) = self.frequency_range.unwrap_or((400, 6000));
        let new_value = (self.target_max_mhz as i32 + delta).clamp(min as i32, max as i32);
        self.target_max_mhz = new_value as u32;
    }

    pub fn apply_cpu_max(&mut self) {
        let result = overclock::set_cpu_max_mhz(self.target_max_mhz);
        self.log_result(result);
    }

    pub fn change_nvidia_power(&mut self, delta: i32) {
        let (min, max) = self
            .nvidia
            .map(|info| (info.power_min_w, info.power_max_w))
            .unwrap_or((50, 600));
        let new_value = (self.nvidia_power_w as i32 + delta).clamp(min as i32, max.max(min) as i32);
        self.nvidia_power_w = new_value as u32;
    }

    pub fn change_nvidia_clock(&mut self, delta: i32) {
        let max = self.nvidia.map(|info| info.clock_max_mhz).unwrap_or(3000);
        let new_value = (self.nvidia_clock_mhz as i32 + delta)
            .clamp(NVIDIA_CLOCK_FLOOR_MHZ as i32, max.max(NVIDIA_CLOCK_FLOOR_MHZ) as i32);
        self.nvidia_clock_mhz = new_value as u32;
    }

    pub fn apply_nvidia_power(&mut self) {
        let result = overclock::nvidia_set_power_limit(self.nvidia_power_w);
        self.log_result(result);
    }

    pub fn apply_nvidia_clock(&mut self) {
        let result = overclock::nvidia_lock_clocks(NVIDIA_CLOCK_FLOOR_MHZ, self.nvidia_clock_mhz);
        self.log_result(result);
    }
}

// ---------------------------------------------------------------------------
// Rendering: the header stays on top, everything below it scrolls.
// Each section is built by its own file in `components/`.
// ---------------------------------------------------------------------------

impl Render for StressApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tests_row = div()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(self.cpu_panel(cx))
            .child(self.ram_panel(cx))
            .child(self.gpu_panel(cx));

        let settings_row = div()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(self.safety_panel(cx))
            .child(self.tuning_panel(cx))
            .child(self.nvidia_panel(cx));

        let content = div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .child(self.metrics())
            .child(tests_row)
            .child(settings_row)
            .child(self.log_panel());

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::TEXT))
            .child(self.header(cx))
            .child(
                div()
                    .id("scroll-area")
                    .flex_1()
                    .min_h_0() // Lets this area shrink so the scrolling works.
                    .overflow_y_scroll()
                    .child(content),
            )
    }
}
