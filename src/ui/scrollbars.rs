use gpui_kit::{component::scroll::Scrollbar, prelude::*, *};

/// Anchor the track to the viewport rather than its scrolling content.
/// The shared platform theme also styles the editor's built-in scrollbars.
pub(super) fn vertical(id: &'static str, handle: &ScrollHandle) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .child(Scrollbar::vertical(handle).id(id))
}

#[cfg(test)]
mod tests {
    use super::vertical;
    use crate::scrollbar_theme::{self, DesktopPreferences, Platform};
    use gpui_kit::{
        Context, InteractiveElement, IntoElement, Modifiers, MouseButton, ParentElement, Point,
        Render, ScrollHandle, StatefulInteractiveElement, Styled, TestAppContext, Window,
        component::{Theme, scroll::ScrollbarMode},
        div, point, px,
    };

    struct ScrollView {
        left: ScrollHandle,
        right: ScrollHandle,
    }

    fn pane(handle: &ScrollHandle, id: &'static str) -> impl IntoElement {
        div()
            .relative()
            .w(px(200.))
            .h(px(160.))
            .child(
                div()
                    .id((id, 0_usize))
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(handle)
                    .child(div().h(px(1000.)).w_full()),
            )
            .child(vertical(id, handle))
    }

    impl Render for ScrollView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .child(pane(&self.left, "left"))
                .child(pane(&self.right, "right"))
        }
    }

    #[gpui_kit::test]
    fn dragging_one_scrollbar_preserves_the_other_viewport(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            Theme::set_scrollbar_mode(ScrollbarMode::Always, cx);
        });
        let left = ScrollHandle::new();
        let right = ScrollHandle::new();
        let (_, cx) = cx.add_window_view({
            let left = left.clone();
            let right = right.clone();
            move |_, _| ScrollView { left, right }
        });
        for platform in [Platform::Mac, Platform::Windows, Platform::Linux] {
            cx.update(|window, cx| {
                scrollbar_theme::apply(platform, false, DesktopPreferences::default(), cx);
                left.set_offset(Point::default());
                window.draw(cx).clear(cx);
            });
            let width = left.bounds().size.width;
            cx.simulate_mouse_down(
                point(px(194.), px(12.)),
                MouseButton::Left,
                Modifiers::none(),
            );
            cx.simulate_mouse_move(
                point(px(194.), px(100.)),
                Some(MouseButton::Left),
                Modifiers::none(),
            );
            cx.simulate_mouse_up(
                point(px(194.), px(100.)),
                MouseButton::Left,
                Modifiers::none(),
            );
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert!(left.offset().y < px(-100.));
            assert_eq!(right.offset(), Point::default());
            assert_eq!(left.bounds().size.width, width);
            // Desktop preference changes must not reset the document position.
            let offset = left.offset();
            cx.update(|window, cx| {
                scrollbar_theme::apply(
                    platform,
                    true,
                    DesktopPreferences {
                        overlay_scrolling: Some(true),
                        ..DesktopPreferences::default()
                    },
                    cx,
                );
                window.draw(cx).clear(cx);
            });
            assert_eq!(left.offset(), offset);
            assert_eq!(left.bounds().size.width, width);
        }
    }
}
