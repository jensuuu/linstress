//! Tuning section: CPU frequency limit, governor and NVIDIA reset (needs root).

use gpui::{div, prelude::*, Context, Div};

use super::button::{button, step_button};
use super::panel::{description, panel};
use super::stepper::stepper;
use crate::app::StressApp;
use crate::overclock;
use crate::theme::{ACCENT, SURFACE_HIGH};

impl StressApp {
    pub fn tuning_panel(&self, cx: &mut Context<Self>) -> Div {
        let range_text = match self.frequency_range {
            Some((min, max)) => format!("Hardware range: {min} – {max} MHz"),
            None => "cpufreq is not available on this system".to_string(),
        };

        panel("Tuning (needs root)")
            .flex_1()
            .child(description(range_text))
            .child(stepper(
                "CPU max frequency",
                format!("{} MHz", self.target_max_mhz),
                step_button("mhz-minus", "–").on_click(cx.listener(|this, _, _, cx| {
                    this.change_target_mhz(-100);
                    cx.notify();
                })),
                step_button("mhz-plus", "+").on_click(cx.listener(|this, _, _, cx| {
                    this.change_target_mhz(100);
                    cx.notify();
                })),
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(button("apply-max", "Apply max frequency", ACCENT).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.apply_cpu_max();
                            cx.notify();
                        }),
                    ))
                    .child(button("gov-performance", "Performance", SURFACE_HIGH).on_click(
                        cx.listener(|this, _, _, cx| {
                            let result = overclock::set_cpu_governor("performance");
                            this.log_result(result);
                            cx.notify();
                        }),
                    ))
                    .child(button("gov-powersave", "Powersave", SURFACE_HIGH).on_click(
                        cx.listener(|this, _, _, cx| {
                            let result = overclock::set_cpu_governor("powersave");
                            this.log_result(result);
                            cx.notify();
                        }),
                    ))
                    .child(button("nvidia-reset", "Reset NVIDIA clocks", SURFACE_HIGH).on_click(
                        cx.listener(|this, _, _, cx| {
                            let result = overclock::nvidia_reset_clocks();
                            this.log_result(result);
                            cx.notify();
                        }),
                    )),
            )
    }
}
