//! A setting row: label on the left, `–  value  +` on the right.

use gpui::{div, prelude::*, px, rgb, Div, FontWeight, Stateful};

use crate::theme::TEXT_DIM;

pub fn stepper(label: &'static str, value: String, minus: Stateful<Div>, plus: Stateful<Div>) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_2()
        .child(div().text_sm().text_color(rgb(TEXT_DIM)).child(label))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(minus)
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .w(px(100.0))
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .child(value),
                )
                .child(plus),
        )
}
