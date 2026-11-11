//! RAM stress test section.

use gpui::{prelude::*, Context, Div};

use super::button::{step_button, toggle_button};
use super::panel::{description, panel, status_line};
use super::stepper::stepper;
use crate::app::{StressApp, RAM_STEP_MB};
use crate::theme::{DANGER, GOOD};

impl StressApp {
    pub fn ram_panel(&self, cx: &mut Context<Self>) -> Div {
        let status = self.ram_test.as_ref().map(|test| {
            if test.allocation_failed() {
                status_line("Could not allocate the memory".into(), DANGER)
            } else if test.errors() > 0 {
                status_line(
                    format!("{} errors in {} cycles: memory is unstable", test.errors(), test.cycles()),
                    DANGER,
                )
            } else {
                status_line(format!("{} cycles, 0 errors", test.cycles()), GOOD)
            }
        });

        panel("RAM stress")
            .flex_1()
            .child(description("Writes patterns to memory and checks that every value is correct."))
            .child(stepper(
                "Memory to test",
                format!("{} MB", self.ram_size_mb),
                step_button("ram-minus", "–").on_click(cx.listener(|this, _, _, cx| {
                    this.change_ram_size(-RAM_STEP_MB);
                    cx.notify();
                })),
                step_button("ram-plus", "+").on_click(cx.listener(|this, _, _, cx| {
                    this.change_ram_size(RAM_STEP_MB);
                    cx.notify();
                })),
            ))
            .children(status)
            .child(
                toggle_button("ram-toggle", self.ram_test.is_some(), "Start RAM test").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.toggle_ram_test();
                        cx.notify();
                    }),
                ),
            )
    }
}
