//! The row of live value cards (usage, frequency, RAM, temperatures).

use gpui::{div, prelude::*, Div};

use super::metric_card::metric_card;
use crate::app::StressApp;
use crate::theme::{ACCENT, DANGER, GOOD, TEXT, TEXT_DIM, WARNING};

/// Picks a color for a temperature: green is fine, yellow is warm, red is near the limit.
fn temperature_color(temperature: Option<f32>, limit: f32) -> u32 {
    match temperature {
        None => TEXT_DIM,
        Some(t) if t < limit - 15.0 => GOOD,
        Some(t) if t < limit => WARNING,
        Some(_) => DANGER,
    }
}

fn format_optional(value: Option<f32>, unit: &str) -> String {
    match value {
        Some(v) => format!("{v:.0} {unit}"),
        None => "N/A".to_string(),
    }
}

impl StressApp {
    pub fn metrics(&self) -> Div {
        let limit = self.temperature_limit;

        let (ram_text, ram_fraction) = match self.memory {
            Some((total, available)) => {
                let used = total.saturating_sub(available);
                (
                    format!("{:.1} / {:.1} GB", used as f32 / 1024.0, total as f32 / 1024.0),
                    Some(used as f32 / total.max(1) as f32),
                )
            }
            None => ("N/A".to_string(), None),
        };

        div()
            .flex()
            .flex_wrap()
            .gap_3()
            .child(metric_card(
                "CPU USAGE",
                format!("{:.0} %", self.cpu_usage),
                ACCENT,
                Some(self.cpu_usage / 100.0),
            ))
            .child(metric_card(
                "CPU FREQUENCY",
                format_optional(self.cpu_frequency_mhz, "MHz"),
                TEXT,
                match (self.cpu_frequency_mhz, self.frequency_range) {
                    (Some(now), Some((_, max))) => Some(now / max as f32),
                    _ => None,
                },
            ))
            .child(metric_card("RAM USED", ram_text, ACCENT, ram_fraction))
            .child(metric_card(
                "CPU TEMPERATURE",
                format_optional(self.cpu_temperature, "°C"),
                temperature_color(self.cpu_temperature, limit),
                self.cpu_temperature.map(|t| t / limit),
            ))
            .child(metric_card(
                "GPU TEMPERATURE",
                format_optional(self.gpu_temperature, "°C"),
                temperature_color(self.gpu_temperature, limit),
                self.gpu_temperature.map(|t| t / limit),
            ))
    }
}
