//! A card that shows one live value, with an optional progress bar.

use gpui::{div, prelude::*, px, relative, rgb, Div, FontWeight};

use crate::theme::{BORDER, SURFACE, SURFACE_HIGH, TEXT_DIM};

/// A thin horizontal progress bar. `fraction` goes from 0.0 to 1.0.
pub fn progress_bar(fraction: f32, color: u32) -> Div {
    div()
        .w_full()
        .h(px(6.0))
        .rounded_full()
        .bg(rgb(SURFACE_HIGH))
        .child(
            div()
                .h_full()
                .w(relative(fraction.clamp(0.0, 1.0)))
                .rounded_full()
                .bg(rgb(color)),
        )
}

/// A card with a title, a big value and an optional progress bar.
pub fn metric_card(title: &'static str, value: String, color: u32, fraction: Option<f32>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .flex_1()
        .min_w(px(160.0))
        .p_4()
        .rounded_xl()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(BORDER))
        .child(div().text_xs().text_color(rgb(TEXT_DIM)).child(title))
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(color))
                .child(value),
        )
        .children(fraction.map(|f| progress_bar(f, color)))
}
