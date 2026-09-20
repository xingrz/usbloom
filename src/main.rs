mod ui;
use gpui_kit::{
    component::{Root, Theme, ThemeMode, TitleBar},
    *,
};
use ui::{Explorer, Quit};

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);
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
                            traffic_light_position: Some(point(
                                px(20.),
                                (ui::TITLEBAR_HEIGHT - px(14.)) / 2.,
                            )),
                        }),
                        app_id: Some("me.xingrz.usbloom".into()),
                        window_min_size: Some(size(px(960.), px(640.))),
                        ..TitleBar::window_options()
                    },
                    |window, cx| {
                        let view = cx.new(|cx| Explorer::new(window, cx));
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("could not open USBloom");
            })
            .detach();
        });
}
