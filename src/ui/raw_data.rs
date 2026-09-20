use gpui_kit::{
    component::{
        ActiveTheme,
        input::{Copy, Editor, EditorState, SelectAll},
    },
    prelude::*,
    *,
};

/// A bounded editor viewport keeps selection and scrolling independent of the
/// full descriptor length. Its buffer is replaced only when the JSON changes.
#[derive(IntoElement)]
pub(super) struct RawData(pub SharedString);

impl RenderOnce for RawData {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let editor = window.use_keyed_state("raw-editor", cx, |window, cx| {
            EditorState::new(window, cx)
                .line_number(false)
                .soft_wrap(false)
                .default_value(self.0.clone())
        });
        if editor.read(cx).value() != self.0 {
            editor.update(cx, |state, cx| state.set_value(self.0, window, cx));
        }
        div()
            .id("raw-data")
            .flex_1()
            .min_h_0()
            .rounded(px(10.))
            .overflow_hidden()
            .bg(cx.theme().sidebar)
            .child(
                Editor::new(&editor)
                    .h(relative(1.))
                    .readonly(true)
                    .appearance(false)
                    .bordered(false)
                    .text_xs()
                    .aria_label("Raw USB descriptors")
                    .context_menu(|menu, _, _| {
                        menu.menu("Copy", Box::new(Copy))
                            .menu("Select all", Box::new(SelectAll))
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::RawData;
    use gpui_kit::{
        Context, IntoElement, Modifiers, ParentElement, Render, ScrollDelta, ScrollWheelEvent,
        SharedString, Styled, TestAppContext, TouchPhase, Window, div, point, px,
    };

    struct RawView(SharedString);
    impl Render for RawView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(600.))
                .h(px(300.))
                .flex()
                .flex_col()
                .child(RawData(self.0.clone()))
        }
    }

    #[gpui_kit::test]
    fn large_raw_document_remains_copyable_and_readonly_after_scrolling(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let json = format!(
            "[\n{}\n]",
            (0..10_000)
                .map(|index| format!("  {{\"interface\": {index}, \"name\": \"Audio 音频\"}}"))
                .collect::<Vec<_>>()
                .join(",\n")
        );
        let (_, cx) = cx.add_window_view({
            let json = SharedString::from(json.clone());
            move |_, _| RawView(json)
        });
        cx.simulate_click(point(px(50.), px(20.)), Modifiers::none());
        let select_all = if cfg!(target_os = "macos") {
            "cmd-a"
        } else {
            "ctrl-a"
        };
        let copy = if cfg!(target_os = "macos") {
            "cmd-c"
        } else {
            "ctrl-c"
        };
        cx.simulate_keystrokes(select_all);
        // Editing must not replace a selected descriptor buffer.
        cx.simulate_input("not a descriptor");
        for _ in 0..5 {
            cx.simulate_event(ScrollWheelEvent {
                position: point(px(100.), px(100.)),
                delta: ScrollDelta::Pixels(point(px(0.), px(-500.))),
                touch_phase: TouchPhase::Moved,
                modifiers: Modifiers::none(),
            });
        }
        cx.simulate_keystrokes(copy);
        let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
        assert_eq!(copied.as_deref(), Some(json.as_str()));
    }
}
