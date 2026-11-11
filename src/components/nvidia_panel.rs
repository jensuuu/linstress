//! NVIDIA GPU tuning: power limit and core clock lock through `nvidia-smi` (needs root).

use gpui::{div, prelude::*, Context, Div};

use super::button::{button, step_button};
use super::panel::{description, panel};
use super::stepper::stepper;
use crate::app::{StressApp, NVIDIA_CLOCK_STEP_MHZ, NVIDIA_POWER_STEP_W};
use crate::overclock;
use crate::theme::{ACCENT, SURFACE_HIGH};

impl StressApp {
    pub fn nvidia_panel(&self, cx: &mut Context<Self>) -> Div {
        let card = panel("NVIDIA GPU (needs root)").flex_1();

        let Some(info) = self.nvidia else {
            return card.child(description(
                "No NVIDIA GPU found. Needs the NVIDIA driver and nvidia-smi.",
            ));
        };

        card
            .child(description(format!(
                "Power limit range: {} – {} W. Highest core clock: {} MHz.",
                info.power_min_w, info.power_max_w, info.clock_max_mhz
            )))
            .child(stepper(
                "Power limit",
                format!("{} W", self.nvidia_power_w),
                step_button("nv-power-minus", "–").on_click(cx.listener(|this, _, _, cx| {
                    this.change_nvidia_power(-NVIDIA_POWER_STEP_W);
                    cx.notify();
                })),
                step_button("nv-power-plus", "+").on_click(cx.listener(|this, _, _, cx| {
                    this.change_nvidia_power(NVIDIA_POWER_STEP_W);
                    cx.notify();
                })),
            ))
            .child(stepper(
                "Max core clock",
                format!("{} MHz", self.nvidia_clock_mhz),
                step_button("nv-clock-minus", "–").on_click(cx.listener(|this, _, _, cx| {
                    this.change_nvidia_clock(-NVIDIA_CLOCK_STEP_MHZ);
                    cx.notify();
                })),
                step_button("nv-clock-plus", "+").on_click(cx.listener(|this, _, _, cx| {
                    this.change_nvidia_clock(NVIDIA_CLOCK_STEP_MHZ);
                    cx.notify();
                })),
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(button("nv-apply-power", "Apply power limit", ACCENT).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.apply_nvidia_power();
                            cx.notify();
                        }),
                    ))
                    .child(button("nv-apply-clock", "Lock clock", ACCENT).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.apply_nvidia_clock();
                            cx.notify();
                        }),
                    ))
                    .child(button("nv-reset", "Reset clocks", SURFACE_HIGH).on_click(
                        cx.listener(|this, _, _, cx| {
                            let result = overclock::nvidia_reset_clocks();
                            this.log_result(result);
                            cx.notify();
                        }),
                    )),
            )
    }
}
