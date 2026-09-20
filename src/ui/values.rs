use gpui_kit::{
    base::{SelectableText, TextSelection, TextSelectionHandle},
    component::{ActiveTheme, native_menu::NativeMenu},
    prelude::*,
    *,
};
use std::cell::Cell;

#[derive(Action, Clone, PartialEq, serde::Deserialize)]
#[action(namespace = usbloom, no_json)]
pub(super) struct CopyValue {
    pub text: String,
}

// Assign a deterministic reading order to independently selectable values.
// Rebuilding an unchanged view produces the same order across refreshes.
#[derive(Default)]
pub(super) struct Values {
    next: Cell<u64>,
}

impl Values {
    #[track_caller]
    pub(super) fn pill(&self, value: String, cx: &App) -> impl IntoElement {
        div()
            .px(px(10.))
            .py(px(5.))
            .rounded(px(6.))
            .bg(cx.theme().muted)
            .text_color(cx.theme().muted_foreground)
            .text_xs()
            .child(self.text(value))
    }
    #[track_caller]
    pub(super) fn code(&self, value: impl Into<SharedString>, cx: &App) -> Div {
        div()
            .font_family(cx.theme().mono_font_family.clone())
            .child(self.text(value))
    }

    pub(super) fn decoded_value(&self, value: String, cx: &App) -> Div {
        match value.rsplit_once(" · ") {
            Some((name, raw)) if raw.starts_with("0x") => div()
                .flex()
                .items_baseline()
                .flex_wrap()
                .gap(px(4.))
                .child(self.text(name.to_owned()))
                .child("·")
                .child(self.code(raw.to_owned(), cx)),
            _ => div().child(self.text(value)),
        }
    }

    pub(super) fn metric(&self, label: &'static str, value: String, cx: &App) -> impl IntoElement {
        div()
            .id(label)
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(
                self.code(value, cx)
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM),
            )
    }
    pub(super) fn field(&self, label: &'static str, value: String, cx: &App) -> impl IntoElement {
        div()
            .id(label)
            .py(px(13.))
            .border_b_1()
            .border_color(cx.theme().border)
            .flex()
            .gap(px(16.))
            .child(
                div()
                    .w(px(165.))
                    .flex_shrink_0()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(div().flex_1().min_w_0().child(
                if matches!(
                    label,
                    "Serial number"
                        | "Device release"
                        | "Port path"
                        | "Device address"
                        | "Negotiated speed"
                        | "Control packet size"
                ) && value != "Unavailable"
                {
                    self.code(value, cx)
                } else {
                    self.decoded_value(value, cx)
                },
            ))
    }
    /// Only data values join the window selection; labels and controls stay out.
    #[track_caller]
    pub(super) fn text<T: Into<SharedString>>(&self, value: T) -> impl IntoElement + use<T> {
        let order = self.next.get();
        self.next.set(order + 1);
        let value = value.into();
        let id = ElementId::CodeLocation(*std::panic::Location::caller());
        div()
            .id(id.clone())
            .cursor_text()
            .debug_selector(|| format!("value-{order}"))
            .child(RefreshingText {
                value: value.clone(),
                order,
            })
            .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                // Keep the exact value even if focus or selection changes while
                // the native menu is tracking the pointer.
                let selected = TextSelection::selected_text(window, cx);
                let text = if selected.is_empty() {
                    value.to_string()
                } else {
                    selected
                };
                cx.stop_propagation();
                NativeMenu::new()
                    .menu("Copy", Box::new(CopyValue { text }))
                    .show(event.position, window, cx);
            })
    }
}

// Keep the selection handle and its redraw subscription together for exactly
// as long as this value is mounted. The default SelectableText handle does not
// subscribe to selection changes in GPUI Kit 0.6.4.
struct SelectionState {
    handle: TextSelectionHandle,
    _redraw: Subscription,
}

#[derive(IntoElement)]
struct RefreshingText {
    value: SharedString,
    order: u64,
}

impl RenderOnce for RefreshingText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("selection-state", cx, |window, cx| {
            let handle = TextSelectionHandle::new(self.value.clone(), cx);
            let redraw = handle.refresh_window_on_change(window, cx);
            SelectionState {
                handle,
                _redraw: redraw,
            }
        });
        SelectableText::with_handle("text", state.read(cx).handle.clone(), self.value)
            .document_order(self.order)
    }
}

#[cfg(test)]
mod tests {
    use super::Values;
    use gpui_kit::{
        Context, InteractiveElement, IntoElement, Modifiers, MouseButton, ParentElement, Render,
        Styled, TestAppContext, Window,
        base::{TextSelection, TextSelectionLayer},
        div, point, px,
    };
    use std::{cell::Cell, rc::Rc};

    struct SelectionView(Rc<Cell<usize>>);

    impl Render for SelectionView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.0.set(self.0.get() + 1);
            div().size_full().child(TextSelectionLayer).child(
                div()
                    .w(px(300.))
                    .h(px(32.))
                    .child(Values::default().text("0123456789ABCDEF")),
            )
        }
    }

    #[gpui_kit::test]
    fn selection_repaints_during_drag_and_settles_when_idle(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let renders = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let renders = renders.clone();
            move |_, _| SelectionView(renders)
        });
        cx.run_until_parked();
        cx.simulate_mouse_down(point(px(1.), px(12.)), MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(
            point(px(40.), px(12.)),
            Some(MouseButton::Left),
            Modifiers::none(),
        );
        let first = cx.update(TextSelection::selected_text);
        let before = renders.get();

        // Stay inside the same text hitbox: no hover, scan, timer, or forced
        // draw may mask a missing redraw subscription.
        cx.simulate_mouse_move(
            point(px(90.), px(12.)),
            Some(MouseButton::Left),
            Modifiers::none(),
        );
        assert!(
            renders.get() > before,
            "dragging within a value must schedule a repaint"
        );
        let second = cx.update(TextSelection::selected_text);
        assert!(!first.is_empty());
        assert!(second.starts_with(&first));
        assert!(
            second.len() > first.len(),
            "selection must track the pointer before mouse-up"
        );

        cx.simulate_mouse_up(
            point(px(90.), px(12.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        let settled = renders.get();
        cx.run_until_parked();
        assert_eq!(
            renders.get(),
            settled,
            "selection must not cause an idle repaint loop"
        );

        cx.simulate_click(point(px(250.), px(100.)), Modifiers::none());
        assert!(cx.update(TextSelection::selected_text).is_empty());
        assert!(
            renders.get() > settled,
            "clearing selection must repaint too"
        );
    }

    struct MixedSelectionView(Rc<Cell<usize>>);

    impl Render for MixedSelectionView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.0.set(self.0.get() + 1);
            assert!(
                self.0.get() < 100,
                "mixed text must not create a redraw loop"
            );
            div()
                .size_full()
                .text_size(px(20.))
                .child(TextSelectionLayer)
                .child(
                    div()
                        .id("mixed")
                        .w(px(300.))
                        .h(px(40.))
                        .child(Values::default().decoded_value("Audio · 0x01".into(), cx)),
                )
        }
    }

    #[gpui_kit::test]
    fn mixed_fonts_select_in_both_directions_without_redraw_loops(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let renders = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let renders = renders.clone();
            move |_, _| MixedSelectionView(renders)
        });
        cx.run_until_parked();
        let name = cx.debug_bounds("value-0").unwrap();
        let raw = cx.debug_bounds("value-1").unwrap();
        let left = point(name.left() + px(1.), name.center().y);
        let right = point(raw.right() - px(1.), raw.center().y);
        for (start, end) in [(left, right), (right, left)] {
            cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
            cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::none());
            let selected = cx.update(TextSelection::selected_text);
            assert!(
                selected.contains("Audio"),
                "missing decoded name: {selected:?}"
            );
            assert!(selected.contains("0x01"), "missing raw code: {selected:?}");
            cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
            let settled = renders.get();
            cx.run_until_parked();
            assert_eq!(renders.get(), settled);
        }
    }
}
