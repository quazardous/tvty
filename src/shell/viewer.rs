//! The image viewer: an image of a thread over the whole window, fitted at
//! first. The wheel or + / − zoom (about the pointer for the wheel), 1 shows
//! it at its real size, 0 or a double click fits it again, a drag moves it,
//! ← → go to the thread's other images, Esc closes. A bar at the bottom
//! gives its name, size and zoom, and opens it in the default application
//! when aiball keeps it on this machine.

use crate::ui::Named as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::images::Picture;
use crate::theme::p;
use crate::ui::buttons;

/// Room kept around a fitted image, and for the bar.
const MARGIN: f32 = 32.;
const BAR: f32 = 36.;

pub(super) struct Viewer {
    pictures: Vec<Picture>,
    index: usize,
    /// `None`: fitted to the window; else the scale, 1 is the real size.
    zoom: Option<f32>,
    /// How far the image was dragged from the centre.
    offset: Point<Pixels>,
    /// Where a drag started, and the offset then.
    drag: Option<(Point<Pixels>, Point<Pixels>)>,
}

impl Viewer {
    pub(super) fn new(pictures: Vec<Picture>, index: usize) -> Self {
        let index = index.min(pictures.len().saturating_sub(1));
        Self { pictures, index, zoom: None, offset: Point::default(), drag: None }
    }

    fn picture(&self) -> Option<&Picture> {
        self.pictures.get(self.index)
    }

    /// The scale that fits the image in `area`, never above its real size.
    fn fit(&self, area: Size<Pixels>) -> f32 {
        let Some(p) = self.picture() else { return 1. };
        let w = (f32::from(area.width) - 2. * MARGIN).max(1.) / p.width.max(1) as f32;
        let h = (f32::from(area.height) - 2. * MARGIN - BAR).max(1.) / p.height.max(1) as f32;
        w.min(h).min(1.)
    }

    fn scale(&self, area: Size<Pixels>) -> f32 {
        self.zoom.unwrap_or_else(|| self.fit(area))
    }

    fn set_zoom(&mut self, zoom: Option<f32>) {
        self.zoom = zoom.map(|z| z.clamp(0.05, 16.));
        if self.zoom.is_none() {
            self.offset = Point::default();
        }
    }

    fn go(&mut self, step: isize) {
        let n = self.pictures.len() as isize;
        if n > 1 {
            self.index = (self.index as isize + step).rem_euclid(n) as usize;
            self.set_zoom(None);
        }
    }
}

impl Shell {
    /// Keys while the viewer is up; answers whether it took the key.
    pub(super) fn viewer_key(&mut self, key: &str, window: &Window, cx: &mut Context<Self>) -> bool {
        let area = window.viewport_size();
        let Some(viewer) = self.viewer.as_mut() else { return false };
        match key {
            "escape" => self.viewer = None,
            "+" | "=" => {
                let s = viewer.scale(area);
                viewer.set_zoom(Some(s * 1.25));
            }
            "-" | "_" => {
                let s = viewer.scale(area);
                viewer.set_zoom(Some(s / 1.25));
            }
            "1" => viewer.set_zoom(Some(1.)),
            "0" => viewer.set_zoom(None),
            "left" => viewer.go(-1),
            "right" => viewer.go(1),
            _ => return false,
        }
        cx.notify();
        true
    }

    pub(super) fn viewer_view(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let viewer = self.viewer.as_ref()?;
        let picture = viewer.picture()?.clone();
        let area = window.viewport_size();
        let scale = viewer.scale(area);
        let (w, h) = (picture.width as f32 * scale, picture.height as f32 * scale);
        let left = (f32::from(area.width) - w) / 2. + f32::from(viewer.offset.x);
        let top = (f32::from(area.height) - BAR - h) / 2. + f32::from(viewer.offset.y);
        let count = viewer.pictures.len();
        // What the text calls it, else its file.
        let name = if picture.alt.trim().is_empty() {
            picture.reference.rsplit('/').next().unwrap_or(&picture.reference).to_string()
        } else {
            picture.alt.clone()
        };
        let path = picture.path.clone();
        let bar = div()
            .absolute()
            .bottom_0()
            .left_0()
            .right_0()
            .h(px(BAR))
            .flex()
            .items_center()
            .gap_4()
            .px_4()
            .bg(p().surface.opacity(0.85))
            .text_sm()
            .text_color(p().muted)
            .child(div().text_color(p().text).truncate().child(name))
            .child(format!("{} × {}", picture.width, picture.height))
            .child(format!("{:.0} %{}", scale * 100., if viewer.zoom.is_none() { crate::t!("misc-fitted") } else { String::new() }))
            .when(count > 1, |d| d.child(format!("{} / {count} · ← →", viewer.index + 1)))
            .child(div().flex_1())
            .child(crate::t!("misc-viewer-hint"))
            .when_some(path, |d, path| {
                d.child(
                    buttons::link("viewer-open", "open")
                        // The system's viewer, whatever the OS.
                        .on_click(move |_, _, cx| cx.open_with_system(std::path::Path::new(&path))),
                )
            });
        Some(
            div()
                .named("viewer")
                .absolute()
                .inset_0()
                .occlude()
                .bg(gpui_kit::black().opacity(0.92))
                .cursor(if viewer.drag.is_some() { CursorStyle::ClosedHand } else { CursorStyle::OpenHand })
                .on_scroll_wheel(cx.listener(|shell, event: &ScrollWheelEvent, window, cx| {
                    let area = window.viewport_size();
                    let Some(viewer) = shell.viewer.as_mut() else { return };
                    let dy = match event.delta {
                        ScrollDelta::Lines(d) => d.y,
                        ScrollDelta::Pixels(d) => f32::from(d.y) / 40.,
                    };
                    if dy == 0. {
                        return;
                    }
                    let before = viewer.scale(area);
                    let after = (before * 1.15f32.powf(dy.signum())).clamp(0.05, 16.);
                    // About the pointer: the point under it stays under it.
                    let centre = point(area.width / 2., (area.height - px(BAR)) / 2.) + viewer.offset;
                    let from = event.position - centre;
                    let k = after / before;
                    viewer.offset = viewer.offset + from - point(from.x * k, from.y * k);
                    viewer.zoom = Some(after);
                    cx.stop_propagation();
                    cx.notify();
                }))
                .on_mouse_down(MouseButton::Left, cx.listener(|shell, event: &MouseDownEvent, _, cx| {
                    if let Some(viewer) = shell.viewer.as_mut() {
                        if event.click_count >= 2 {
                            viewer.set_zoom(None);
                        } else {
                            viewer.drag = Some((event.position, viewer.offset));
                        }
                        cx.notify();
                    }
                }))
                .on_mouse_move(cx.listener(|shell, event: &MouseMoveEvent, _, cx| {
                    if let Some(viewer) = shell.viewer.as_mut() {
                        if let (Some((from, start)), true) = (viewer.drag, event.dragging()) {
                            viewer.offset = start + (event.position - from);
                            cx.notify();
                        }
                    }
                }))
                .on_mouse_up(MouseButton::Left, cx.listener(|shell, _, _, cx| {
                    if let Some(viewer) = shell.viewer.as_mut() {
                        viewer.drag = None;
                        cx.notify();
                    }
                }))
                .child(
                    img(ImageSource::Image(picture.image.clone()))
                        .absolute()
                        .left(px(left))
                        .top(px(top))
                        .w(px(w))
                        .h(px(h)),
                )
                .child(bar)
                .child(
                    buttons::link("viewer-close", crate::t!("tickets-close-full"))
                        .absolute()
                        .top_2()
                        .right_3()
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.viewer = None;
                            cx.notify();
                        })),
                )
                .into_any_element(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Viewer;

    #[test]
    fn an_empty_viewer_is_harmless() {
        let v = Viewer::new(Vec::new(), 3);
        assert!(v.picture().is_none());
        assert_eq!(v.fit(gpui_kit::size(gpui_kit::px(800.), gpui_kit::px(600.))), 1.);
    }
}
