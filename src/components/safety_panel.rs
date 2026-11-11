//! Safety section: the temperature at which all tests stop automatically.

use gpui::{div, prelude::*, rgb, Context, Div};

use super::button::step_button;
use super::panel::{description, panel};
use super::stepper::stepper;
use crate::app::StressApp;
use crate::theme::{WARNING, WARNING_BACKGROUND};

impl StressApp {
    pub fn safety_panel(&self, cx: &mut Context<Self>) -> Div {
        let warning = (!self.has_temperature_sensor()).then(|| {
            div()
                .p_3()
                .rounded_lg()
                .bg(rgb(WARNING_BACKGROUND))
                .border_1()
                .border_color(rgb(WARNING))
                .text_sm()
                .text_color(rgb(WARNING))
                .child("No temperature sensor found. The automatic safety stop is disabled.")
        });

        panel("Safety")
            .flex_1()
            .child(description("Every running test stops when the CPU or GPU reaches this temperature."))
            .child(stepper(
                "Safety temperature",
                format!("{:.0} °C", self.temperature_limit),
                step_button("limit-minus", "–").on_click(cx.listener(|this, _, _, cx| {
                    this.change_temperature_limit(-5.0);
                    cx.notify();
                })),
                step_button("limit-plus", "+").on_click(cx.listener(|this, _, _, cx| {
                    this.change_temperature_limit(5.0);
                    cx.notify();
                })),
            ))
            .children(warning)
        
    }
}
