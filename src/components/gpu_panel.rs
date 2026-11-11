//! GPU stress test section.

use gpui::{prelude::*, Context, Div};

use super::button::toggle_button;
use super::panel::{description, panel, status_line};
use crate::app::StressApp;
use crate::theme::GOOD;

impl StressApp {
    pub fn gpu_panel(&self, cx: &mut Context<Self>) -> Div {
        let status = self.gpu_test.as_ref().map(|test| {
            status_line(format!("{}: {} batches done", test.adapter_name(), test.batches()), GOOD)
        });

        panel("GPU stress")
            .flex_1()
            .child(description("Runs a heavy compute shader through Vulkan on your fastest GPU."))
            .children(status)
            .child(
                toggle_button("gpu-toggle", self.gpu_test.is_some(), "Start GPU test").on_click(
                    cx.listener(|this, _, _, cx| {
                        this.toggle_gpu_test();
                        cx.notify();
                    }),
                ),
            )
    }
}
