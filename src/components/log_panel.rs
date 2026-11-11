//! The list of recent messages at the bottom.

use gpui::{div, prelude::*, rgb, Div};

use super::panel::panel;
use crate::app::StressApp;
use crate::theme::TEXT_DIM;

impl StressApp {
    pub fn log_panel(&self) -> Div {
        let lines = self
            .log
            .iter()
            .map(|line| div().text_sm().text_color(rgb(TEXT_DIM)).child(line.clone()));

        panel("Log").children(lines)
    }
}
