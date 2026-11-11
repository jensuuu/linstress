//! The rounded box used for every section, plus its text helpers.

use gpui::{div, prelude::*, px, rgb, Div, FontWeight, SharedString};

use crate::theme::{BORDER, SURFACE, TEXT_DIM};

/// A rounded box with a heading. Add `.flex_1()` when it should share a row.
pub fn panel(title: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .min_w(px(280.0))
        .rounded_xl()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(BORDER))
        .child(div().text_base().font_weight(FontWeight::SEMIBOLD).child(title))
}

/// Small gray explanation text.
pub fn description(text: impl Into<SharedString>) -> Div {
    div().text_xs().text_color(rgb(TEXT_DIM)).child(text.into())
}

/// One line of colored status text.
pub fn status_line(text: String, color: u32) -> Div {
    div().text_sm().text_color(rgb(color)).child(text)
}
