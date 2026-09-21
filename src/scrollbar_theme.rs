//! Platform policy shared by pane scrollbars and the raw descriptor editor.
use gpui_kit::{
    App, Global, base,
    component::{
        Theme,
        scroll::{ScrollbarMode, ScrollbarMotion, ScrollbarStyles},
    },
    px, rgb, transparent_black,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DesktopPreferences {
    pub overlay_scrolling: Option<bool>,
    pub high_contrast: bool,
    pub reduced_motion: bool,
}
impl Global for DesktopPreferences {}

#[derive(Clone, Copy)]
pub(crate) enum Platform {
    Mac,
    Windows,
    Linux,
}

pub(crate) fn platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Mac
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

pub(crate) fn preferences(cx: &App) -> DesktopPreferences {
    cx.try_global::<DesktopPreferences>()
        .copied()
        .unwrap_or_default()
}

pub(crate) fn mode(
    platform: Platform,
    auto_hide: bool,
    prefs: DesktopPreferences,
) -> ScrollbarMode {
    if prefs.high_contrast {
        return ScrollbarMode::Always;
    }
    match platform {
        Platform::Mac if auto_hide => ScrollbarMode::Scrolling,
        Platform::Windows if auto_hide => ScrollbarMode::Hover,
        Platform::Linux if prefs.overlay_scrolling == Some(true) => ScrollbarMode::Hover,
        _ => ScrollbarMode::Always,
    }
}

/// Apply after Kit projects its palette into the base theme. Editor scrollbars
/// use the base theme directly, so per-pane overrides would miss raw data.
pub(crate) fn sync(cx: &mut App) {
    apply(
        platform(),
        cx.should_auto_hide_scrollbars(),
        preferences(cx),
        cx,
    );
}

pub(crate) fn apply(platform: Platform, auto_hide: bool, prefs: DesktopPreferences, cx: &mut App) {
    let mode = mode(platform, auto_hide, prefs);
    let persistent = matches!(mode, ScrollbarMode::Always);
    let theme = Theme::global(cx);
    let dark = theme.is_dark();
    // Neutral ink and tracks do not assume an Ubuntu, GNOME, or KDE theme.
    let thumb = if prefs.high_contrast {
        theme.foreground
    } else {
        rgb(if dark { 0xa0a0a0 } else { 0x777777 }).into()
    };
    let active: gpui_kit::Hsla = rgb(if dark { 0xd0d0d0 } else { 0x454545 }).into();
    let track = rgb(if dark { 0x282828 } else { 0xf0f0f0 }).into();
    let clear = transparent_black();
    let (width, expanded, radius, inset) = match platform {
        Platform::Mac => (6., 8., 4., 4.),
        Platform::Windows => (if persistent { 6. } else { 2. }, 6., 1., 5.),
        Platform::Linux if persistent => (8., 8., 2., 3.),
        Platform::Linux => (3., 8., 4., 3.),
    };
    let track_width = if matches!(platform, Platform::Linux) {
        14.
    } else {
        16.
    };
    let styles = ScrollbarStyles::default()
        .track(|s| {
            s.width(px(track_width))
                .bg(if persistent { track } else { clear })
                .border_color(clear)
        })
        .track_hover(|s| s.width(px(track_width)).bg(track).border_color(clear))
        .track_active(|s| s.width(px(track_width)).bg(track).border_color(clear))
        .thumb(|s| {
            s.bg(thumb)
                .width(px(width))
                .inset(px(inset))
                .radius(px(radius))
        })
        .thumb_hover(|s| {
            s.bg(active)
                .width(px(expanded))
                .inset(px(inset))
                .radius(px(radius))
        })
        .thumb_active(|s| {
            s.bg(active)
                .width(px(expanded))
                .inset(px(inset))
                .radius(px(radius))
        });
    let base_theme = base::Theme::global_mut(cx);
    let mut scrollbar = base_theme
        .scrollbar
        .clone()
        .with_mode(mode)
        .with_styles(styles);
    if prefs.reduced_motion {
        scrollbar = scrollbar.with_motion(ScrollbarMotion::default());
    }
    base_theme.scrollbar = scrollbar;
}
