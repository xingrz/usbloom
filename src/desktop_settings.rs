//! Read desktop capabilities rather than inferring preferences from a distro.
use crate::scrollbar_theme::DesktopPreferences;
use ashpd::{desktop::settings::Settings, zvariant::OwnedValue};
use futures_lite::StreamExt;
use gpui_kit::App;

const APPEARANCE: &str = "org.freedesktop.appearance";
// Optional backend extension, not part of the cross-desktop portal contract.
const GTK_INTERFACE: &str = "org.gnome.desktop.interface";

fn update(prefs: &mut DesktopPreferences, namespace: &str, key: &str, value: &OwnedValue) -> bool {
    let previous = *prefs;
    match (namespace, key) {
        (GTK_INTERFACE, "overlay-scrolling") => {
            prefs.overlay_scrolling = bool::try_from(value).ok()
        }
        (APPEARANCE, "contrast") => prefs.high_contrast = u32::try_from(value).ok() == Some(1),
        (APPEARANCE, "reduced-motion") => {
            prefs.reduced_motion = u32::try_from(value).ok() == Some(1)
        }
        _ => {}
    }
    *prefs != previous
}

pub(crate) fn observe(cx: &mut App) {
    cx.set_global(DesktopPreferences::default());
    cx.spawn(async move |cx| {
        // Missing portals/settings are normal on minimal desktop sessions.
        let Ok(settings) = Settings::new().await else {
            return;
        };
        // Subscribe before taking the snapshot so startup changes are not lost.
        let Ok(mut events) = settings.receive_setting_changed().await else {
            return;
        };
        let mut prefs = DesktopPreferences::default();
        if let Ok(namespaces) = settings.read_all(&[APPEARANCE, GTK_INTERFACE]).await {
            for (namespace, values) in namespaces {
                for (key, value) in values {
                    update(&mut prefs, &namespace, &key, &value);
                }
            }
        }
        cx.update(|cx| {
            cx.set_global(prefs);
            crate::appearance::sync(None, cx);
        });
        while let Some(event) = events.next().await {
            if update(&mut prefs, event.namespace(), event.key(), event.value()) {
                cx.update(|cx| {
                    cx.set_global(prefs);
                    crate::appearance::sync(None, cx);
                });
            }
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::{APPEARANCE, GTK_INTERFACE, update};
    use crate::scrollbar_theme::DesktopPreferences;

    #[test]
    fn optional_settings_can_change_and_revert_without_losing_other_preferences() {
        let mut prefs = DesktopPreferences::default();
        assert!(update(
            &mut prefs,
            GTK_INTERFACE,
            "overlay-scrolling",
            &true.into()
        ));
        assert!(update(&mut prefs, APPEARANCE, "contrast", &1_u32.into()));
        assert!(!update(&mut prefs, APPEARANCE, "unknown-key", &true.into()));
        assert_eq!(prefs.overlay_scrolling, Some(true));
        assert!(update(
            &mut prefs,
            GTK_INTERFACE,
            "overlay-scrolling",
            &42_u32.into()
        ));
        assert_eq!(prefs.overlay_scrolling, None);
        assert!(prefs.high_contrast);
        assert!(update(&mut prefs, APPEARANCE, "contrast", &42_u32.into()));
        assert!(!prefs.high_contrast);
        assert!(update(
            &mut prefs,
            APPEARANCE,
            "reduced-motion",
            &1_u32.into()
        ));
        assert!(prefs.reduced_motion);
        assert!(update(
            &mut prefs,
            APPEARANCE,
            "reduced-motion",
            &0_u32.into()
        ));
        assert!(!prefs.reduced_motion);
    }
}
