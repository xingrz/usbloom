use gpui_kit::{
    assets::IconName,
    component::{ActiveTheme, Icon, InteractiveElementExt},
    prelude::*,
    *,
};
use std::sync::LazyLock;

pub(super) fn is_gnome() -> bool {
    static GNOME: LazyLock<bool> = LazyLock::new(|| {
        std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .split(':')
            .any(|desktop| desktop.eq_ignore_ascii_case("gnome"))
    });
    *GNOME
}

/// GNOME does not provide server decorations on Wayland. Keep its headerbar
/// proportions for the client-decorated fallback, without changing other WMs.
#[derive(IntoElement)]
pub(super) struct HeaderBar(pub AnyElement);

impl RenderOnce for HeaderBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let dragging = window.use_state(cx, |_, _| false);
        let supported = window.window_controls();
        div()
            .id("linux-headerbar")
            .flex()
            .flex_shrink_0()
            .items_center()
            .h(super::TITLEBAR_HEIGHT)
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .on_double_click(|_, window, _| window.zoom_window())
            .on_mouse_down(MouseButton::Right, |event, window, _| {
                window.show_window_menu(event.position)
            })
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&dragging, |dragging, _, _, _| *dragging = true),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&dragging, |dragging, _, _, _| *dragging = false),
            )
            .on_mouse_down_out(window.listener_for(&dragging, |dragging, _, _, _| {
                *dragging = false;
            }))
            .on_mouse_move(window.listener_for(&dragging, |dragging, _, window, _| {
                if *dragging {
                    *dragging = false;
                    window.start_window_move();
                }
            }))
            .child(self.0)
            .when(!window.is_fullscreen(), |bar| {
                bar.child(
                    div()
                        .id("linux-window-controls")
                        .flex()
                        .flex_shrink_0()
                        .items_center()
                        .gap(px(8.))
                        .pr(px(12.))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_double_click(|_, _, cx| cx.stop_propagation())
                        .when(supported.minimize, |controls| {
                            controls.child(control("minimize", IconName::WindowMinimize, cx))
                        })
                        .when(supported.maximize, |controls| {
                            controls.child(control(
                                "maximize",
                                if window.is_maximized() {
                                    IconName::WindowRestore
                                } else {
                                    IconName::WindowMaximize
                                },
                                cx,
                            ))
                        })
                        .child(control("close", IconName::WindowClose, cx)),
                )
            })
    }
}

fn control(id: &'static str, icon: IconName, cx: &App) -> impl IntoElement {
    div()
        .id(id)
        .size(px(24.))
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(cx.theme().secondary)
        .hover(|style| style.bg(cx.theme().secondary_hover))
        .active(|style| style.bg(cx.theme().secondary_active))
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            match id {
                "minimize" => window.minimize_window(),
                "maximize" => window.zoom_window(),
                _ => window.remove_window(),
            }
        })
        .child(Icon::new(icon).size(px(12.)))
}
