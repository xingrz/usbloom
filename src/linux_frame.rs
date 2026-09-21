use gpui_kit::{component::ActiveTheme, prelude::*, *};

const SHADOW: Pixels = px(20.);
const RADIUS: Pixels = px(12.);
const RESIZE: Pixels = px(4.);

pub fn radius(window: &Window) -> Pixels {
    match window.window_decorations() {
        Decorations::Client { tiling } if !tiling.is_tiled() && !window.is_fullscreen() => RADIUS,
        _ => px(0.),
    }
}

/// The application paints each surface's exposed corners itself. Keeping the
/// root transparent avoids relying on GPUI's rectangular child clipping.
pub struct Frame(pub AnyView);

impl Render for Frame {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Decorations::Client { tiling } = window.window_decorations() else {
            return div().size_full().child(self.0.clone()).into_any_element();
        };
        // Keep this stable across maximize/restore; only the visible insets vary.
        window.set_client_inset(SHADOW);
        let tiling = if window.is_fullscreen() {
            Tiling {
                top: true,
                bottom: true,
                left: true,
                right: true,
            }
        } else {
            tiling
        };
        let insets = Edges {
            top: if tiling.top { px(0.) } else { SHADOW },
            bottom: if tiling.bottom { px(0.) } else { SHADOW },
            left: if tiling.left { px(0.) } else { SHADOW },
            right: if tiling.right { px(0.) } else { SHADOW },
        };
        let radius = radius(window);
        let zones = resize_zones(window.viewport_size(), insets, tiling);
        div()
            .size_full()
            .relative()
            .flex()
            .child(
                div()
                    .size_full()
                    .pt(insets.top)
                    .pb(insets.bottom)
                    .pl(insets.left)
                    .pr(insets.right)
                    .child(
                        div()
                            .size_full()
                            .rounded(radius)
                            .border_color(cx.theme().border)
                            .when(!tiling.top, |view| view.border_t_1())
                            .when(!tiling.bottom, |view| view.border_b_1())
                            .when(!tiling.left, |view| view.border_l_1())
                            .when(!tiling.right, |view| view.border_r_1())
                            .when(!tiling.is_tiled(), |view| {
                                view.shadow(vec![BoxShadow {
                                    color: rgba(0x00000030).into(),
                                    blur_radius: px(12.),
                                    spread_radius: px(-1.),
                                    offset: point(px(0.), px(2.)),
                                    inset: false,
                                }])
                            })
                            .child(self.0.clone()),
                    ),
            )
            .children(
                zones
                    .into_iter()
                    .enumerate()
                    .map(|(index, (bounds, edge, cursor))| {
                        div()
                            .id(("frame-resize", index))
                            .absolute()
                            .left(bounds.origin.x)
                            .top(bounds.origin.y)
                            .w(bounds.size.width)
                            .h(bounds.size.height)
                            .cursor(cursor)
                            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                cx.stop_propagation();
                                window.start_window_resize(edge);
                            })
                    }),
            )
            .into_any_element()
    }
}

fn resize_zones(
    size: Size<Pixels>,
    insets: Edges<Pixels>,
    tiling: Tiling,
) -> Vec<(Bounds<Pixels>, ResizeEdge, CursorStyle)> {
    let left = insets.left;
    let right = size.width - insets.right;
    let top = insets.top;
    let bottom = size.height - insets.bottom;
    let mut zones = Vec::new();
    for (x, y, edge, cursor) in [
        (-1, 0, ResizeEdge::Left, CursorStyle::ResizeLeftRight),
        (1, 0, ResizeEdge::Right, CursorStyle::ResizeLeftRight),
        (0, -1, ResizeEdge::Top, CursorStyle::ResizeUpDown),
        (0, 1, ResizeEdge::Bottom, CursorStyle::ResizeUpDown),
        (
            -1,
            -1,
            ResizeEdge::TopLeft,
            CursorStyle::ResizeUpLeftDownRight,
        ),
        (
            1,
            1,
            ResizeEdge::BottomRight,
            CursorStyle::ResizeUpLeftDownRight,
        ),
        (
            1,
            -1,
            ResizeEdge::TopRight,
            CursorStyle::ResizeUpRightDownLeft,
        ),
        (
            -1,
            1,
            ResizeEdge::BottomLeft,
            CursorStyle::ResizeUpRightDownLeft,
        ),
    ] {
        if (x < 0 && tiling.left)
            || (x > 0 && tiling.right)
            || (y < 0 && tiling.top)
            || (y > 0 && tiling.bottom)
        {
            continue;
        }
        let (x0, x1) = match x {
            -1 => (left - RESIZE, left + RESIZE),
            1 => (right - RESIZE, right + RESIZE),
            _ => (left + RESIZE, right - RESIZE),
        };
        let (y0, y1) = match y {
            -1 => (top - RESIZE, top + RESIZE),
            1 => (bottom - RESIZE, bottom + RESIZE),
            _ => (top + RESIZE, bottom - RESIZE),
        };
        zones.push((
            Bounds::from_corners(point(x0, y0), point(x1, y1)),
            edge,
            cursor,
        ));
    }
    zones
}

#[cfg(test)]
mod tests {
    use super::{SHADOW, resize_zones};
    use gpui_kit::{Edges, Tiling, point, px, size};

    #[test]
    fn resize_zones_leave_controls_and_shadow_clear_and_obey_tiling() {
        let insets = Edges::all(SHADOW);
        let size = size(px(1200.), px(800.));
        let free = resize_zones(size, insets, Tiling::default());
        assert_eq!(
            free.iter()
                .filter(|(bounds, _, _)| bounds.contains(&point(px(20.), px(20.))))
                .count(),
            1
        );
        for point in [point(px(5.), px(5.)), point(px(1160.), px(44.))] {
            assert!(!free.iter().any(|(bounds, _, _)| bounds.contains(&point)));
        }
        let maximized = Tiling {
            top: true,
            bottom: true,
            left: true,
            right: true,
        };
        assert!(resize_zones(size, Edges::all(px(0.)), maximized).is_empty());
        assert_eq!(
            resize_zones(size, insets, Tiling::default()).len(),
            free.len()
        );
    }
}
