//! The bar at the top: title, "Stop all" button and status badge.

use gpui::{div, prelude::*, rgb, Context, Div, FontWeight};

use super::button::button;
use super::panel::description;
use crate::app::StressApp;
use crate::theme::{BORDER, DANGER, GOOD, SURFACE, SURFACE_HIGH, WARNING};

impl StressApp {
    pub fn header(&self, cx: &mut Context<Self>) -> Div {
        let (status_text, status_color) = if self.any_running() {
            ("● Test running", WARNING)
        } else {
            ("● Idle", GOOD)
        };

        let stop_all_button = self.any_running().then(|| {
            button("stop-all", "Stop all", DANGER).on_click(cx.listener(|this, _, _, cx| {
                this.stop_all();
                cx.notify();
            }))
        });

        div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_3()
            .px_6()
            .py_4()
            .border_b_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child("Stress Test & Tuner"),
                    )
                    .child(description("CPU, RAM and GPU stress tests with frequency control")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .children(stop_all_button)
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .bg(rgb(SURFACE_HIGH))
                            .text_sm()
                            .text_color(rgb(status_color))
                            .child(status_text),
                    ),
            )
    }
}
