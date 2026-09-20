use gpui_kit::{
    component::{Root, Theme, ThemeMode, button::*},
    prelude::*,
    *,
};

struct Explorer {
    status: String,
    scanning: bool,
}

impl Explorer {
    fn scan(&mut self, cx: &mut Context<Self>) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.status = "Reading connected devices…".into();
        cx.notify();
        let scan = cx.background_executor().spawn(async {
            cyme::profiler::get_spusb_with_extra()
                .map(|profile| profile.flattened_devices().len())
                .map_err(|error| error.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = scan.await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |state, cx| {
                    state.scanning = false;
                    state.status = match result {
                        Ok(count) => format!("{count} connected devices"),
                        Err(error) => format!("Could not read devices: {error}"),
                    };
                    cx.notify();
                });
            }
        })
        .detach();
    }
}

impl Render for Explorer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xf7f8fa))
            .text_color(rgb(0x222c34))
            .font_family(".AppleSystemUIFont")
            .child(
                div()
                    .h(px(68.))
                    .px(px(28.))
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(rgb(0xe4e8ec))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("USBloom"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(20.))
                    .child(
                        div()
                            .size(px(64.))
                            .rounded(px(20.))
                            .bg(rgb(0xe0eee9))
                            .text_color(rgb(0x287e68))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_3xl()
                            .child("U"),
                    )
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("A closer look at your devices."),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x72808c))
                            .child(self.status.clone()),
                    )
                    .child(
                        Button::new("scan")
                            .primary()
                            .label(if self.scanning {
                                "Scanning…"
                            } else {
                                "Scan devices"
                            })
                            .on_click(cx.listener(|this, _, _, cx| this.scan(cx))),
                    ),
            )
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(1120.), px(760.)), cx);
            cx.spawn(async move |cx| {
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("USBloom".into()),
                            ..Default::default()
                        }),
                        app_id: Some("me.xingrz.usbloom".into()),
                        window_min_size: Some(size(px(800.), px(560.))),
                        ..Default::default()
                    },
                    |window, cx| {
                        let view = cx.new(|_| Explorer {
                            status: "Explore your USB devices, down to every endpoint.".into(),
                            scanning: false,
                        });
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("could not open USBloom");
            })
            .detach();
        });
}
