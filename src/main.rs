#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod appearance;
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
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("could not open USBloom");
            })
            .detach();
        });
}
