use gpui_kit::{component::ActiveTheme, prelude::*, *};

/// Keep caption controls at the top edge, independently of toolbar height.
/// Windows handles these hit regions, including the maximize snap flyout.
pub(super) fn title_bar(content: impl IntoElement, window: &Window, cx: &App) -> impl IntoElement {
    let supported = window.window_controls();
    div()
        .flex()
        .flex_shrink_0()
        .h(super::TITLEBAR_HEIGHT)
        .border_b_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .h_full()
                .window_control_area(WindowControlArea::Drag)
                .child(content),
        )
        .when(!window.is_fullscreen(), |bar| {
            bar.child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .h(px(32.))
                    .when(supported.minimize, |controls| {
                        controls.child(caption_button(
                            "minimize",
                            "\u{e921}",
                            WindowControlArea::Min,
                            window,
                            cx,
                        ))
                    })
                    .when(supported.maximize, |controls| {
                        controls.child(caption_button(
                            "maximize",
                            if window.is_maximized() {
                                "\u{e923}"
                            } else {
                                "\u{e922}"
                            },
                            WindowControlArea::Max,
                            window,
                            cx,
                        ))
                    })
                    .child(caption_button(
                        "close",
                        "\u{e8bb}",
                        WindowControlArea::Close,
                        window,
                        cx,
                    )),
            )
        })
}

fn caption_button(
    id: &'static str,
    glyph: &'static str,
    area: WindowControlArea,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let close = matches!(area, WindowControlArea::Close);
    div()
        .id(id)
        .w(px(46.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .font_family("Segoe Fluent Icons")
        .text_size(px(10.))
        .line_height(px(10.))
        .text_color(if window.is_window_active() {
            cx.theme().foreground
        } else {
            cx.theme().muted_foreground
        })
        .hover(|style| {
            if close {
                style.bg(rgb(0xc42b1c)).text_color(rgb(0xffffff))
            } else {
                style.bg(cx.theme().secondary_hover)
            }
        })
        .active(|style| {
            if close {
                style.bg(rgb(0xa82418)).text_color(rgb(0xffffff))
            } else {
                style.bg(cx.theme().secondary_active)
            }
        })
        .window_control_area(area)
        .child(glyph)
}
