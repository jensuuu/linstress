//! CPU stress test section.

use gpui::{prelude::*, Context, Div};

use super::button::{step_button, toggle_button};
use super::panel::{description, panel};
use super::stepper::stepper;
use crate::app::StressApp;

impl StressApp {
    pub fn cpu_panel(&self, cx: &mut Context<Self>) -> Div {
        panel("CPU stress")
            .flex_1()
            .child(description("Heavy floating-point math on the chosen number of threads."))
            .child(stepper(
                "Worker threads",
                self.cpu_threads.to_string(),
                step_button("cpu-threads-minus", "–").on_click(cx.listener(|this, _, _, cx| {
                    this.change_cpu_threads(-1);
                    cx.notify();
                })),
                step_button("cpu-threads-plus", "+").on_click(cx.listener(|this, _, _, cx| {
                    this.change_cpu_threads(1);
                    cx.notify();
                })),
            ))
            .child(
                toggle_button("cpu-toggle", self.cpu_test.is_some(), "Start CPU test").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.toggle_cpu_test();
                        cx.notify();
                    }),
                ),
            )
    }
}
