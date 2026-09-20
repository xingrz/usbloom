use cyme::usb::{Configuration, Interface};
use gpui_kit::{
    assets::IconName,
    component::{
        ActiveTheme, Disableable, Icon, InteractiveElementExt, Selectable, Sizable, TitleBar,
        button::*,
        input::{Input, InputEvent, InputState},
        tooltip::Tooltip,
    },
    prelude::*,
    *,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use usbloom::inventory::{self, DeviceRow, Snapshot};

gpui_kit::actions!(usbloom, [Quit, Refresh, Find, OpenSnapshot, SaveSnapshot]);
pub const TITLEBAR_HEIGHT: Pixels = px(64.);

const INK: u32 = 0x23332f;
const MUTED: u32 = 0x77857f;
const LINE: u32 = 0xe5eae7;
const ACCENT: u32 = 0x297f68;
const TINT: u32 = 0xe5f1eb;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Interfaces,
    Device,
    Raw,
}

pub struct Explorer {
    snapshot: Option<Arc<Snapshot>>,
    rows: Vec<DeviceRow>,
    selected: Option<String>,
    collapsed: HashSet<String>,
    config: Option<u8>,
    alternatives: HashMap<(u8, u8), u8>,
    tab: Tab,
    search: Entity<InputState>,
    focus: FocusHandle,
    detail_scroll: ScrollHandle,
    scanning: bool,
    live: bool,
    source: Option<String>,
    error: Option<String>,
    notice: String,
    epoch: u64,
    notice_task: Option<Task<()>>,
    _search_subscription: Subscription,
    _watch: Task<()>,
}

impl Explorer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Find a device…"));
        let subscription = cx.subscribe(&search, |_, _, _: &InputEvent, cx| cx.notify());
        let watch = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(5)).await;
                let Some(this) = this.upgrade() else {
                    break;
                };
                this.update(cx, |state, cx| {
                    if state.live && state.source.is_none() {
                        state.scan(cx);
                    }
                });
            }
        });
        let modifier = if cfg!(target_os = "macos") {
            "cmd"
        } else {
            "ctrl"
        };
        cx.bind_keys([
            KeyBinding::new(&format!("{modifier}-r"), Refresh, None),
            KeyBinding::new(&format!("{modifier}-f"), Find, None),
            KeyBinding::new(&format!("{modifier}-o"), OpenSnapshot, None),
            KeyBinding::new(&format!("{modifier}-s"), SaveSnapshot, None),
        ]);
        let mut view = Self {
            snapshot: None,
            rows: vec![],
            selected: None,
            collapsed: HashSet::new(),
            config: None,
            alternatives: HashMap::new(),
            tab: Tab::Interfaces,
            search,
            focus: cx.focus_handle(),
            detail_scroll: ScrollHandle::new(),
            scanning: false,
            live: true,
            source: None,
            error: None,
            notice: String::new(),
            epoch: 0,
            notice_task: None,
            _search_subscription: subscription,
            _watch: watch,
        };
        view.focus.focus(window, cx);
        view.scan(cx);
        view
    }

    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.rows = snapshot.rows();
        // Preserve a disconnected selection instead of jumping to another device.
        if self.selected.is_none() {
            self.selected = self
                .rows
                .iter()
                .find(|row| !row.has_children)
                .or(self.rows.first())
                .map(|row| row.key.clone());
        }
        self.snapshot = Some(Arc::new(snapshot));
    }

    fn scan(&mut self, cx: &mut Context<Self>) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.epoch += 1;
        let epoch = self.epoch;
        let job = cx
            .background_executor()
            .spawn(async { Snapshot::capture() });
        cx.spawn(async move |this, cx| {
            let result = job.await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |state, cx| {
                    if state.epoch != epoch {
                        return;
                    }
                    state.scanning = false;
                    match result {
                        Ok(snapshot) => {
                            state.apply_snapshot(snapshot);
                            state.error = None;
                        }
                        Err(error) => state.error = Some(format!("Scan failed. {error}")),
                    }
                    cx.notify();
                });
            }
        })
        .detach();
        cx.notify();
    }

    fn show_notice(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.notice = message.into();
        self.notice_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(3)).await;
            let _ = this.update(cx, |state, cx| {
                state.notice.clear();
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.source = None;
        self.notice.clear();
        self.scan(cx);
    }

    fn select(&mut self, key: String, cx: &mut Context<Self>) {
        if self.selected.as_ref() != Some(&key) {
            self.selected = Some(key);
            self.config = None;
            self.alternatives.clear();
            self.detail_scroll.set_offset(point(px(0.), px(0.)));
        }
        cx.notify();
    }

    fn open(&mut self, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open snapshot".into()),
        });
        cx.spawn(async move |this, cx| {
            let path = match prompt.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) => return,
                _ => {
                    let _ = this.update(cx, |state, cx| {
                        state.error = Some("Could not open the file picker.".into());
                        cx.notify();
                    });
                    return;
                }
            };
            let Some(path) = path else {
                return;
            };
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let result = cx
                .background_executor()
                .spawn(async move { Snapshot::load(&path) })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(snapshot) => {
                        state.epoch += 1;
                        state.scanning = false;
                        if !snapshot
                            .rows()
                            .iter()
                            .any(|row| state.selected.as_ref() == Some(&row.key))
                        {
                            state.selected = None;
                        }
                        state.config = None;
                        state.alternatives.clear();
                        state.detail_scroll.set_offset(point(px(0.), px(0.)));
                        state.apply_snapshot(snapshot);
                        state.source = Some(name);
                        state.error = None;
                        state.notice.clear();
                    }
                    Err(error) => state.error = Some(format!("Could not open snapshot. {error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        let folder = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        let prompt = cx.prompt_for_new_path(&folder, Some("USBloom.snapshot.json"));
        cx.spawn(async move |this, cx| {
            let path = match prompt.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) => return,
                _ => {
                    let _ = this.update(cx, |state, cx| {
                        state.error = Some("Could not open the save dialog.".into());
                        cx.notify();
                    });
                    return;
                }
            };
            let result = cx
                .background_executor()
                .spawn(async move { snapshot.save(&path) })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(()) => state.show_notice("Snapshot saved", cx),
                    Err(error) => state.error = Some(format!("Could not save snapshot. {error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .h(TITLEBAR_HEIGHT)
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_between()
            .pl(px(if cfg!(target_os = "macos") { 96. } else { 20. }))
            .pr(px(24.))
            .border_b_1()
            .border_color(rgb(LINE))
            .bg(rgb(0xffffff))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .size(px(28.))
                            .rounded(px(9.))
                            .bg(rgb(ACCENT))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(IconName::Usb)
                                    .size(px(20.))
                                    .text_color(rgb(0xffffff)),
                            ),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_lg()
                            .child("USBloom"),
                    ),
            )
            .child(
                div()
                    .id("toolbar-actions")
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_double_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        Button::new("open")
                            .ghost()
                            .icon(IconName::FolderOpen)
                            .label("Open")
                            .tooltip("Open a saved snapshot")
                            .on_click(cx.listener(|this, _, _, cx| this.open(cx))),
                    )
                    .child(
                        Button::new("save")
                            .ghost()
                            .icon(IconName::Download)
                            .label("Save snapshot")
                            .disabled(self.snapshot.is_none())
                            .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                    )
                    .when(self.source.is_none(), |view| {
                        let label = if self.live {
                            "Pause automatic refresh"
                        } else {
                            "Resume automatic refresh"
                        };
                        view.child(
                            Button::new("auto-refresh")
                                .ghost()
                                .icon(if self.live {
                                    IconName::Pause
                                } else {
                                    IconName::Play
                                })
                                .selected(!self.live)
                                .accessibility_label(label)
                                .tooltip(label)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.live = !this.live;
                                    if this.live {
                                        this.scan(cx);
                                    }
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        Button::new("refresh")
                            .outline()
                            .icon(IconName::RefreshCw)
                            .label(if self.source.is_some() {
                                "Connected devices"
                            } else {
                                "Refresh"
                            })
                            .disabled(self.scanning)
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                    ),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search.read(cx).value().to_string();
        let visible = inventory::visible_rows(&self.rows, &query, &self.collapsed);
        let mut tree = div()
            .id("device-tree")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px(px(12.))
            .pb(px(16.));
        let mut last_bus = String::new();
        for row in &visible {
            if row.bus_key != last_bus {
                tree = tree.child(
                    div()
                        .px(px(12.))
                        .pt(px(20.))
                        .pb(px(9.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_color(rgb(MUTED))
                        .text_xs()
                        .child(Icon::new(IconName::Monitor).size(px(13.)))
                        .child(format!("USB BUS {}", row.device.location_id.bus)),
                );
                last_bus = row.bus_key.clone();
            }
            let selected = self.selected.as_ref() == Some(&row.key);
            let key = row.key.clone();
            let toggle_key = row.key.clone();
            let expanded = !self.collapsed.contains(&row.key) || !query.is_empty();
            let indent = row.depth.min(5) as f32 * 14.;
            let d = &row.device;
            let name = if d.name.is_empty() {
                "Unnamed device".to_string()
            } else {
                d.name.clone()
            };
            tree = tree.child(
                div()
                    .id(SharedString::from(format!("row-{}", row.key)))
                    .role(Role::Button)
                    .aria_label(format!("{name}, {}", inventory::ids(d)))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(58.))
                    .px(px(10.))
                    .ml(px(indent))
                    .mb(px(3.))
                    .rounded(px(9.))
                    .cursor_pointer()
                    .bg(rgb(if selected { TINT } else { 0xf4f6f5 }))
                    .hover(|style| style.bg(rgb(if selected { TINT } else { 0xecefec })))
                    .on_click(cx.listener(move |this, _, _, cx| this.select(key.clone(), cx)))
                    .child(if row.has_children {
                        Button::new(SharedString::from(format!("toggle-{}", row.key)))
                            .ghost()
                            .small()
                            .icon(if expanded {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .tooltip(if expanded {
                                "Collapse hub"
                            } else {
                                "Expand hub"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                if !this.collapsed.remove(&toggle_key) {
                                    this.collapsed.insert(toggle_key.clone());
                                }
                                cx.notify();
                            }))
                            .into_any_element()
                    } else {
                        div().w(px(8.)).into_any_element()
                    })
                    .child(
                        Icon::new(if d.base_class_code() == Some(9) {
                            IconName::Network
                        } else {
                            IconName::Usb
                        })
                        .size(px(18.))
                        .text_color(rgb(if selected {
                            ACCENT
                        } else {
                            MUTED
                        })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(if selected {
                                        FontWeight::SEMIBOLD
                                    } else {
                                        FontWeight::MEDIUM
                                    })
                                    .truncate()
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .truncate()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .child(inventory::ids(d)),
                            ),
                    ),
            );
        }
        if visible.is_empty() {
            tree = tree.child(div().p(px(20.)).text_sm().text_color(rgb(MUTED)).child(
                if self.scanning && self.snapshot.is_none() {
                    "Reading devices…"
                } else if query.is_empty() {
                    "No connected devices"
                } else {
                    "No matching devices"
                },
            ));
        }
        div()
            .w(px(324.))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(0xf4f6f5))
            .border_r_1()
            .border_color(rgb(LINE))
            .child(
                div().p(px(20.)).pb(px(8.)).child(
                    Input::new(&self.search)
                        .aria_label("Find a device")
                        .prefix(Icon::new(IconName::Search).size(px(16.)))
                        .cleanable(true),
                ),
            )
            .child(tree)
    }

    fn detail(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self
            .rows
            .iter()
            .find(|row| self.selected.as_ref() == Some(&row.key))
        else {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.))
                .child(
                    Icon::new(IconName::Usb)
                        .size(px(40.))
                        .text_color(rgb(ACCENT)),
                )
                .child(div().text_xl().child(if self.selected.is_some() {
                    "Device disconnected"
                } else {
                    "Your devices, in detail."
                }))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(MUTED))
                        .child(if self.selected.is_some() {
                            "Reconnect it, or select another device."
                        } else {
                            "Select a device to explore its interfaces and endpoints."
                        }),
                )
                .into_any_element();
        };
        let d = &row.device;
        let key = row.key.clone();
        let raw = serde_json::to_string_pretty(d).unwrap_or_default();
        let raw_copy = raw.clone();
        let configs = d
            .extra
            .as_ref()
            .map(|extra| extra.configurations.as_slice())
            .unwrap_or_default();
        let selected_config = configs
            .iter()
            .find(|config| Some(config.number) == self.config)
            .or_else(|| configs.iter().find(|config| config.active))
            .or(configs.first());
        let mut content = div()
            .id("details-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.detail_scroll)
            .p(px(30.))
            .flex()
            .flex_col()
            .gap(px(24.));
        content = content.child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(row.bus.clone())
                        .child("/")
                        .child("Port")
                        .child(code(d.port_path().to_string(), cx)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(16.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_2xl()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(d.name.clone()),
                        )
                        .child(
                            div()
                                .size(px(48.))
                                .flex_shrink_0()
                                .rounded(px(14.))
                                .bg(rgb(TINT))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Icon::new(if row.has_children {
                                        IconName::Network
                                    } else {
                                        IconName::Usb
                                    })
                                    .size(px(25.))
                                    .text_color(rgb(ACCENT)),
                                ),
                        ),
                )
                .child(
                    div().text_sm().text_color(rgb(MUTED)).child(
                        d.manufacturer
                            .clone()
                            .unwrap_or_else(|| "Manufacturer unavailable".into()),
                    ),
                ),
        );
        content = content.child(
            div()
                .flex()
                .gap(px(8.))
                .flex_wrap()
                .child(pill(inventory::device_class(d)))
                .child(pill(inventory::speed(d))),
        );
        if let Some(error) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.read_errors.get(&key))
        {
            content = content.child(
                div()
                    .p(px(12.))
                    .rounded(px(8.))
                    .bg(rgb(0xfff3dc))
                    .text_color(rgb(0x87682c))
                    .text_sm()
                    .child(format!("Some descriptors could not be read. {error}")),
            );
        }
        content = content.child(
            div()
                .flex()
                .gap(px(28.))
                .pb(px(20.))
                .border_b_1()
                .border_color(rgb(LINE))
                .child(metric("VENDOR ID", inventory::hex16(d.vendor_id), cx))
                .child(metric("PRODUCT ID", inventory::hex16(d.product_id), cx))
                .child(metric(
                    "USB VERSION",
                    d.bcd_usb
                        .map_or_else(|| "Unavailable".into(), |v| v.to_string()),
                    cx,
                )),
        );
        let mut tabs = div().flex().gap(px(6.)).items_center();
        for (tab, label) in [
            (Tab::Interfaces, "Interfaces"),
            (Tab::Device, "Device details"),
            (Tab::Raw, "Raw data"),
        ] {
            tabs = tabs.child(
                Button::new(label)
                    .ghost()
                    .label(label)
                    .selected(self.tab == tab)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.tab = tab;
                        cx.notify();
                    })),
            );
        }
        content = content.child(tabs);
        match self.tab {
            Tab::Interfaces => {
                if let Some(config) = selected_config {
                    let mut config_header =
                        div().flex().items_center().justify_between().gap(px(8.));
                    let mut choices = div().flex().gap(px(6.));
                    for config in configs {
                        let number = config.number;
                        if configs.len() == 1 {
                            choices = choices.child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(format!("Configuration {number}")),
                            );
                            continue;
                        }
                        choices = choices.child(
                            Button::new(("config", number as usize))
                                .ghost()
                                .small()
                                .label(format!("Configuration {number}"))
                                .selected(number == selected_config.unwrap().number)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.config = Some(number);
                                    cx.notify();
                                })),
                        );
                    }
                    config_header = config_header.child(choices).child(
                        div().text_xs().text_color(rgb(MUTED)).child(format!(
                            "{} max · {}",
                            config.max_power,
                            if config.active { "Active" } else { "Inactive" }
                        )),
                    );
                    content = content.child(config_header);
                    for (number, variants) in inventory::interfaces(config) {
                        let alt = self.alternatives.get(&(config.number, number)).copied();
                        let interface = variants
                            .iter()
                            .copied()
                            .find(|i| Some(i.alt_setting) == alt)
                            .or_else(|| variants.iter().copied().find(|i| i.active))
                            .unwrap_or(variants[0]);
                        content = content.child(self.interface_card(
                            config,
                            &variants,
                            interface,
                            d.extra.as_ref().and_then(|e| e.negotiated_speed.as_ref()),
                            cx,
                        ));
                    }
                    if config.interfaces.is_empty() {
                        content = content.child(empty("No interface descriptors were returned."));
                    }
                } else {
                    content = content.child(empty(
                        "Configuration descriptors are unavailable for this device.",
                    ));
                }
            }
            Tab::Device => {
                let fields = vec![
                    (
                        "Manufacturer",
                        d.manufacturer
                            .clone()
                            .unwrap_or_else(|| "Unavailable".into()),
                    ),
                    (
                        "Serial number",
                        d.serial_num.clone().unwrap_or_else(|| "Unavailable".into()),
                    ),
                    (
                        "Device release",
                        d.bcd_device
                            .map_or_else(|| "Unavailable".into(), |v| v.to_string()),
                    ),
                    (
                        "Device class",
                        inventory::decoded(d.base_class_code(), d.class_name()),
                    ),
                    (
                        "Subclass",
                        inventory::decoded(d.sub_class, d.sub_class_name()),
                    ),
                    (
                        "Protocol",
                        inventory::decoded(d.protocol, d.protocol_name()),
                    ),
                    ("Port path", d.port_path().to_string()),
                    ("Device address", d.location_id.number.to_string()),
                    ("Negotiated speed", inventory::speed(d)),
                    ("Control packet size", inventory::control_packet_size(d)),
                ];
                let mut list = div().flex_shrink_0().flex().flex_col();
                for (label, value) in fields {
                    list = list.child(field(label, value, cx));
                }
                content = content.child(list);
            }
            Tab::Raw => {
                content = content.child(
                    div().flex().items_center().justify_end().child(
                        Button::new("copy-raw")
                            .ghost()
                            .small()
                            .icon(IconName::Copy)
                            .label("Copy JSON")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(raw_copy.clone()));
                                this.show_notice("JSON copied", cx);
                                cx.notify();
                            })),
                    ),
                );
                content = content.child(
                    div()
                        .p(px(16.))
                        .bg(rgb(0xf4f6f5))
                        .rounded(px(10.))
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_xs()
                        .children(raw.lines().map(|line| div().child(line.to_owned()))),
                );
            }
        }
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .child(content)
            .into_any_element()
    }

    fn interface_card(
        &self,
        config: &Configuration,
        variants: &[&Interface],
        interface: &Interface,
        speed: Option<&cyme::usb::Speed>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let config_number = config.number;
        let number = interface.number;
        let mut alternatives = div().flex().gap(px(4.));
        for variant in variants {
            let alt = variant.alt_setting;
            if variants.len() == 1 {
                alternatives = alternatives.child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(format!("Alt {alt}")),
                );
                continue;
            }
            alternatives = alternatives.child(
                Button::new(SharedString::from(format!(
                    "alt-{config_number}-{number}-{alt}"
                )))
                .ghost()
                .small()
                .label(format!("Alt {alt}"))
                .selected(alt == interface.alt_setting)
                .tooltip("View this alternate setting; does not change the device")
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.alternatives.insert((config_number, number), alt);
                    cx.notify();
                })),
            );
        }
        let mut card = div()
            .flex_shrink_0()
            .border_1()
            .border_color(rgb(LINE))
            .rounded(px(12.))
            .overflow_hidden()
            .child(
                div()
                    .p(px(16.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .size(px(30.))
                                    .rounded(px(9.))
                                    .bg(rgb(0xf1f4f2))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_sm()
                                    .text_color(rgb(ACCENT))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .child(format!("{number:02}")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(3.))
                                    .child(
                                        div().text_sm().font_weight(FontWeight::SEMIBOLD).child(
                                            interface
                                                .sub_class_name()
                                                .filter(|_| interface.sub_class != 0)
                                                .or(interface.class_name())
                                                .unwrap_or("Vendor specific")
                                                .to_string(),
                                        ),
                                    )
                                    .child(div().text_xs().text_color(rgb(MUTED)).child(format!(
                                        "Interface {number} · {}",
                                        interface.class_name().unwrap_or("Unassigned class")
                                    ))),
                            ),
                    )
                    .child(alternatives),
            )
            .child(
                div()
                    .px(px(16.))
                    .pb(px(12.))
                    .flex()
                    .items_baseline()
                    .gap(px(20.))
                    .flex_wrap()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(4.))
                            .child("Class")
                            .child(code(format!("0x{:02X}", u8::from(interface.class)), cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(4.))
                            .child("Subclass")
                            .child(code(format!("0x{:02X}", interface.sub_class), cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .flex_wrap()
                            .gap(px(4.))
                            .child("Protocol")
                            .child(decoded_value(
                                inventory::decoded(
                                    Some(interface.protocol),
                                    interface.protocol_name(),
                                ),
                                cx,
                            )),
                    ),
            );
        for endpoint in &interface.endpoints {
            let input = endpoint.address.address & 0x80 != 0;
            let timing =
                usbloom::descriptors::interval(&endpoint.transfer_type, speed, endpoint.interval);
            let help = format!(
                "{} {} {} bInterval: {}. {}",
                if input {
                    "IN: device to host."
                } else {
                    "OUT: host to device."
                },
                usbloom::descriptors::transfer_help(&endpoint.transfer_type),
                if matches!(endpoint.transfer_type, cyme::usb::TransferType::Isochronous) {
                    format!(
                        "{} synchronization; {} usage.",
                        endpoint.sync_type, endpoint.usage_type
                    )
                } else {
                    String::new()
                },
                endpoint.interval,
                timing
            );
            card = card.child(
                div()
                    .id(SharedString::from(format!(
                        "endpoint-{config_number}-{number}-{}",
                        endpoint.address.address
                    )))
                    .tooltip(move |window, cx| Tooltip::new(help.clone()).build(window, cx))
                    .px(px(16.))
                    .py(px(12.))
                    .border_t_1()
                    .border_color(rgb(LINE))
                    .flex()
                    .items_center()
                    .gap(px(14.))
                    .child(
                        Icon::new(if input {
                            IconName::ArrowDownLeft
                        } else {
                            IconName::ArrowUpRight
                        })
                        .size(px(17.))
                        .text_color(rgb(ACCENT)),
                    )
                    .child(
                        div()
                            .w(px(82.))
                            .flex_shrink_0()
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!(
                                "EP {} {}",
                                endpoint.address.number,
                                if input { "IN" } else { "OUT" }
                            )),
                    )
                    .child(
                        div()
                            .w(px(42.))
                            .flex_shrink_0()
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(format!("0x{:02X}", endpoint.address.address)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .child(endpoint.transfer_type.to_string()),
                    )
                    .child(code(format!("{} B", endpoint.max_packet_size()), cx).text_sm())
                    .child(
                        div()
                            .w(px(88.))
                            .text_right()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .when(timing.starts_with(|c: char| c.is_ascii_digit()), |view| {
                                view.font_family(cx.theme().mono_font_family.clone())
                            })
                            .child(timing),
                    ),
            );
        }
        if interface.endpoints.is_empty() {
            card = card.child(
                div()
                    .px(px(16.))
                    .pb(px(16.))
                    .text_sm()
                    .text_color(rgb(MUTED))
                    .child("No endpoints"),
            );
        }
        card
    }
}

impl Render for Explorer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .text_color(rgb(INK))
            .text_sm()
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &Refresh, _, cx| this.refresh(cx)))
            .on_action(cx.listener(|this, _: &Find, window, cx| {
                this.search.update(cx, |input, cx| input.focus(window, cx))
            }))
            .on_action(cx.listener(|this, _: &OpenSnapshot, _, cx| this.open(cx)))
            .on_action(cx.listener(|this, _: &SaveSnapshot, _, cx| this.save(cx)))
            .child(self.toolbar(cx))
            .when_some(self.error.clone(), |view, error| {
                view.child(
                    div()
                        .p(px(10.))
                        .bg(rgb(0xfff1dc))
                        .text_color(rgb(0x875d24))
                        .child(error),
                )
            })
            .when_some(self.source.clone(), |view, source| {
                view.child(
                    div()
                        .px(px(24.))
                        .py(px(8.))
                        .bg(rgb(TINT))
                        .text_color(rgb(ACCENT))
                        .child(format!("Snapshot · {source}")),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.sidebar(cx))
                    .child(self.detail(cx)),
            )
            .when(!self.notice.is_empty(), |view| {
                view.child(
                    div()
                        .id("notice")
                        .role(Role::Status)
                        .aria_label(self.notice.clone())
                        .absolute()
                        .bottom(px(20.))
                        .right(px(24.))
                        .px(px(14.))
                        .py(px(10.))
                        .rounded(px(8.))
                        .bg(rgb(INK))
                        .text_color(rgb(0xffffff))
                        .shadow_sm()
                        .text_sm()
                        .child(self.notice.clone()),
                )
            })
    }
}

fn pill(value: String) -> impl IntoElement {
    div()
        .px(px(10.))
        .py(px(5.))
        .rounded(px(6.))
        .bg(rgb(0xf1f5f2))
        .text_color(rgb(0x577064))
        .text_xs()
        .child(value)
}
fn code(value: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .font_family(cx.theme().mono_font_family.clone())
        .child(value.into())
}

fn decoded_value(value: String, cx: &App) -> Div {
    match value.rsplit_once(" · ") {
        Some((name, raw)) if raw.starts_with("0x") => div()
            .flex()
            .items_baseline()
            .flex_wrap()
            .gap(px(4.))
            .child(name.to_owned())
            .child("·")
            .child(code(raw.to_owned(), cx)),
        _ => div().child(value),
    }
}

fn metric(label: &'static str, value: String, cx: &App) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(7.))
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
        .child(code(value, cx).text_lg().font_weight(FontWeight::MEDIUM))
}
fn field(label: &'static str, value: String, cx: &Context<Explorer>) -> impl IntoElement {
    let copy = value.clone();
    div()
        .py(px(13.))
        .border_b_1()
        .border_color(rgb(LINE))
        .flex()
        .gap(px(16.))
        .child(
            div()
                .w(px(165.))
                .flex_shrink_0()
                .text_color(rgb(MUTED))
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
                code(value, cx)
            } else {
                decoded_value(value, cx)
            },
        ))
        .child(
            Button::new(label)
                .ghost()
                .small()
                .icon(IconName::Copy)
                .accessibility_label(format!("Copy {label}"))
                .tooltip(format!("Copy {label}"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()));
                    this.show_notice(format!("{label} copied"), cx);
                    cx.notify();
                })),
        )
}
fn empty(message: &'static str) -> impl IntoElement {
    div()
        .p(px(22.))
        .rounded(px(10.))
        .bg(rgb(0xf6f8f6))
        .text_color(rgb(MUTED))
        .child(message)
}
