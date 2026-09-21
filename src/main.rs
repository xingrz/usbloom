#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod appearance;
#[cfg(any(target_os = "linux", all(unix, test)))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod desktop_settings;
#[cfg(any(target_os = "linux", test))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod linux_frame;
mod scrollbar_theme;
mod ui;
use gpui_kit::{
    component::{Root, TitleBar},
    *,
};
use ui::{Explorer, Quit};

#[cfg(target_os = "macos")]
fn traffic_light_position() -> Point<Pixels> {
    point(px(20.), (ui::TITLEBAR_HEIGHT - px(14.)) / 2.)
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            appearance::sync(None, cx);
            #[cfg(target_os = "linux")]
            desktop_settings::observe(cx);
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.set_menus([Menu::new("USBloom").items([MenuItem::action("Quit USBloom", Quit)])]);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
            cx.spawn(async move |cx| {
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("USBloom".into()),
                            appears_transparent: true,
                            #[cfg(target_os = "macos")]
                            traffic_light_position: Some(traffic_light_position()),
                            #[cfg(not(target_os = "macos"))]
                            traffic_light_position: None,
                        }),
                        app_id: Some("me.xingrz.usbloom".into()),
                        #[cfg(target_os = "linux")]
                        window_decorations: Some(WindowDecorations::Server),
                        window_min_size: Some(size(px(960.), px(640.))),
                        ..TitleBar::window_options()
                    },
                    |window, cx| {
                        // AppKit lays out its standard buttons again after the
                        // window is shown. Align them after that initial layout.
                        #[cfg(target_os = "macos")]
                        window.on_next_frame(|window, _| {
                            window.set_traffic_light_position(traffic_light_position());
                        });
                        appearance::sync(Some(window), cx);
                        window
                            .observe_window_appearance(|window, cx| {
                                appearance::sync(Some(window), cx);
                            })
                            .detach();
                        let view = cx.new(|cx| Explorer::new(window, cx));
                        #[cfg(target_os = "linux")]
                        let view = cx.new(|_| linux_frame::Frame(view.into()));
                        cx.new(|cx| {
                            let root = Root::new(view, window, cx);
                            #[cfg(target_os = "linux")]
                            let root = root.bordered(false).bg(transparent_black());
                            root
                        })
                    },
                )
                .expect("could not open USBloom");
            })
            .detach();
        });
}
