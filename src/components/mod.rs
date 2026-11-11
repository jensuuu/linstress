//! All pieces of the user interface.
//!
//! Small building blocks (stateless functions):
//!   button, stepper, metric_card, panel
//!
//! Sections of the window (each one is a method on `StressApp`):
//!   header, metrics, cpu_panel, ram_panel, gpu_panel,
//!   safety_panel, tuning_panel, nvidia_panel, log_panel

// Building blocks
pub mod button;
pub mod metric_card;
pub mod panel;
pub mod stepper;

// Sections
mod cpu_panel;
mod gpu_panel;
mod header;
mod log_panel;
mod metrics;
mod nvidia_panel;
mod ram_panel;
mod safety_panel;
mod tuning_panel;
