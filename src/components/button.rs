//! Clickable buttons. Add `.on_click(...)` after calling these functions.

use gpui::{div, prelude::*, px, rgb, Div, FontWeight, SharedString, Stateful};

use crate::theme::{ACCENT, DANGER, GOOD, SURFACE_HIGH, TEXT};

/// A regular button with a colored background.
pub fn button(id: &'static str, label: impl Into<SharedString>, background: u32) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .px_4()
        .py_2()
        .rounded_lg()
        .bg(rgb(background))
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(0xffffff))
        .cursor_pointer()
        .hover(|style| style.opacity(0.85))
        .child(label.into())
}

/// A green "Start" button that turns into a red "Stop" button while running.
pub fn toggle_button(id: &'static str, running: bool, start_label: &'static str) -> Stateful<Div> {
    if running {
        button(id, "Stop", DANGER)
    } else {
        button(id, start_label, GOOD)
    }
}

/// A small square button used for  –  and  +.
pub fn step_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(40.0))
        .h(px(34.0))
        .rounded_lg()
        .bg(rgb(SURFACE_HIGH))
        .text_color(rgb(TEXT))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(ACCENT)))
        .child(label)
}
