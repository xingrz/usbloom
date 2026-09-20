use gpui_kit::{App, Window, component::Theme, rgb};

/// Keep native controls, Kit widgets, and device content on the same palette.
pub fn sync(window: Option<&mut Window>, cx: &mut App) {
    Theme::sync_system_appearance(window, cx);
    let theme = Theme::global_mut(cx);
    let dark = theme.is_dark();
    let color = |light, dark_color| rgb(if dark { dark_color } else { light }).into();
    theme.background = color(0xffffff, 0x191f1d);
    theme.foreground = color(0x23332f, 0xe3eae6);
    theme.muted = color(0xf1f5f2, 0x242e29);
    theme.muted_foreground = color(0x6b7972, 0x9eafa5);
    theme.border = color(0xe5eae7, 0x35403a);
    theme.sidebar = color(0xf4f6f5, 0x141a17);
    theme.sidebar_foreground = theme.foreground;
    theme.sidebar_border = theme.border;
    theme.accent = color(0xe5f1eb, 0x263e33);
    theme.accent_foreground = color(0x297f68, 0x8bd5b5);
    theme.primary = theme.accent_foreground;
    theme.primary_hover = color(0x226e59, 0xa2e1c5);
    theme.primary_active = color(0x1a5948, 0x73c5a3);
    theme.primary_foreground = color(0xffffff, 0x12261c);
    theme.secondary = theme.muted;
    theme.secondary_foreground = theme.foreground;
    theme.secondary_hover = color(0xecefec, 0x303c35);
    theme.secondary_active = theme.accent;
    theme.selection = color(0xb8ddcd, 0x365e4d);
    theme.input = theme.border;
    theme.ring = theme.accent_foreground;
    theme.popover = color(0xffffff, 0x242d28);
    theme.popover_foreground = theme.foreground;
    theme.title_bar = theme.background;
    theme.title_bar_border = theme.border;
    theme.warning = color(0xfff3dc, 0x3c3120);
    theme.warning_foreground = color(0x87682c, 0xe6c58b);
    Theme::sync_base(cx);
    cx.refresh_windows();
}
