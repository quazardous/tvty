//! The usage arrow in the title bar ([`crate::usage`]): an arrow, red and
//! up when the subscription goes faster than the pace that would use it up
//! right at the window's end, green and down when slower, deeper as the gap
//! grows; its figure; then a small drawing of what the figure says (the
//! quota left going down to the wall, the gap in points, the recent
//! speed…). A click says the gap another way, figure and drawing, until
//! tvty restarts; the tip draws the three that run in time, on one clock.

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::*;

use super::{Shell, press_kept};
use crate::theme::p;
use crate::ui::buttons;
use crate::usage::{Chart, Display, History};

/// The arrow's colour at no gap: a hint, deeper up to [`crate::usage::FULL_AT`].
const FAINTEST: f32 = 0.4;

/// The steady pace moves with the clock (a third of a point a minute over
/// five hours): the arrow follows it while no reading comes. Each time, the
/// newest reading is kept, for the curve.
const REDRAW_EVERY: std::time::Duration = std::time::Duration::from_secs(60);

impl Shell {
    pub(super) fn follow_usage_pace(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                let shown = this.update(cx, |shell, cx| {
                    if let Some(usage) = crate::usage::freshest(shell.board.bars.values()).cloned() {
                        let now = crate::status::now();
                        crate::config::update::<History>(cx, |history| history.take(&usage, now));
                        cx.notify();
                    }
                });
                if shown.is_err() {
                    break;
                }
                cx.background_executor().timer(REDRAW_EVERY).await;
            }
        })
        .detach();
    }

    /// None until a loop read the usage from its Claude.
    pub(super) fn usage_arrow(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let now = crate::status::now();
        let usage = crate::usage::freshest(self.board.bars.values())?;
        let pace = crate::usage::pace(usage, now)?;
        let chart = crate::usage::chart(crate::config::get::<History>(cx), usage, now);
        let display = self.usage_display.unwrap_or_else(|| self.applied.usage.display());
        let (arrow, colour) = if pace.above() { ("↑", p().danger) } else { ("↓", p().success) };
        let depth = FAINTEST + (1. - FAINTEST) * pace.heat().abs() as f32;
        let label = format!("{arrow} {}", crate::usage::said(&pace, chart.as_ref(), display, now));
        let tip = crate::usage::tip(usage, chart.as_ref(), now);
        let large = chart.clone();
        Some(
            press_kept(buttons::bare("usage-arrow", Some(label.clone())))
                .gap_1()
                .child(label)
                .children(chart.map(|chart| drawn(chart, Drawing::of(display), now).w(px(64.)).h(px(28.))))
                .tooltip(move |window, cx| {
                    let (large, tip) = (large.clone(), tip.clone());
                    Tooltip::element(move |_, _| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(large.clone().map(|chart| drawings(chart, now)))
                            .children(tip.lines().map(|line| div().child(line.to_string())))
                    })
                    .build(window, cx)
                })
                .text_xs()
                .text_color(colour.opacity(depth))
                .flex_none()
                .mr_2()
                .on_click(cx.listener(move |shell, _, _, cx| {
                    cx.stop_propagation();
                    shell.usage_display = Some(display.next());
                    cx.notify();
                })),
        )
    }
}

/// A time of day, local: the drawings' ends.
fn clock(at: u64) -> String {
    chrono::DateTime::from_timestamp(at as i64, 0).map(|at| at.with_timezone(&chrono::Local).format("%H:%M").to_string()).unwrap_or_default()
}

/// What a drawing shows: each figure has its own.
#[derive(Clone, Copy, PartialEq)]
enum Drawing {
    /// What is used (a bar) and where the steady pace is now (a tick).
    Ratio,
    /// The gap to the pace in points, from the window's start, around 0.
    Points,
    /// The quota left going down against the pace, on to the wall.
    Wall,
    /// The recent speed against the pace (the line at 1), a scope's trace.
    Trend,
}

impl Drawing {
    fn of(display: Display) -> Self {
        match display {
            Display::Ratio => Drawing::Ratio,
            Display::Points => Drawing::Points,
            Display::Wall => Drawing::Wall,
            Display::Trend => Drawing::Trend,
        }
    }
}

/// The tip's drawings: the three that run in time, one above the other on
/// the same clock, from the window's start to its reset.
fn drawings(chart: Chart, now: u64) -> Div {
    let (from, to) = (clock(chart.start), clock(chart.resets_at));
    let row = |drawing: Drawing, name: String| {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(div().w(px(44.)).flex_none().text_xs().text_color(p().muted).child(name))
            .child(drawn(chart.clone(), drawing, now).w(px(320.)).h(px(64.)))
    };
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(Drawing::Wall, crate::t!("usage-drawing-wall")))
        .child(row(Drawing::Points, crate::t!("usage-drawing-points")))
        .child(row(Drawing::Trend, crate::t!("usage-drawing-trend")))
        .child(div().flex().ml(px(52.)).w(px(320.)).justify_between().text_xs().text_color(p().muted).child(from).child(to))
}

/// A drawing on black, as a scope's screen: it reads alike in every theme.
fn drawn(chart: Chart, drawing: Drawing, now: u64) -> Div {
    div().flex_none().rounded_sm().border_1().border_color(p().border).bg(gpui_kit::rgb(0x000000)).child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    let mut pen = Pen { chart: &chart, bounds, now, width: px(2.), window };
                    match drawing {
                        Drawing::Ratio => pen.ratio(),
                        Drawing::Points => pen.points(),
                        Drawing::Wall => pen.wall(),
                        Drawing::Trend => pen.trend(),
                    }
                });
            },
        )
        .size_full(),
    )
}

/// The trend's scale: the speed shown up to this many times the pace.
const FASTEST: f64 = 2.5;

/// The gap's scale: at least this many points either side of 0.
const WIDEST_GAP: f64 = 20.;

struct Pen<'a, 'w> {
    chart: &'a Chart,
    bounds: Bounds<Pixels>,
    now: u64,
    width: Pixels,
    window: &'w mut Window,
}

impl Pen<'_, '_> {
    /// Where `at` is, from the window's start to its reset.
    fn x(&self, at: u64) -> Pixels {
        let length = self.length();
        self.bounds.left() + self.bounds.size.width * ((at.saturating_sub(self.chart.start) as f64 / length).min(1.) as f32)
    }

    /// Where `share` (0 to 1) is, from the bottom.
    fn y(&self, share: f64) -> Pixels {
        self.bounds.bottom() - self.bounds.size.height * share.clamp(0., 1.) as f32
    }

    fn length(&self) -> f64 {
        self.chart.resets_at.saturating_sub(self.chart.start).max(1) as f64
    }

    /// Percent used by now at the steady pace.
    fn steady(&self, at: u64) -> f64 {
        at.saturating_sub(self.chart.start) as f64 / self.length() * 100.
    }

    fn line(&mut self, points: &[Point<Pixels>], colour: Hsla, dashed: bool, width: Pixels) {
        let mut path = PathBuilder::stroke(width);
        if dashed {
            path = path.dash_array(&[px(2.), px(2.)]);
        }
        path.add_polygon(points, false);
        if let Ok(path) = path.build() {
            self.window.paint_path(path, colour);
        }
    }

    /// Filled, `colour` faint: what is drawn over it stays seen.
    fn area(&mut self, points: &[Point<Pixels>], colour: Hsla) {
        self.fill(points, colour.opacity(0.35));
    }

    fn fill(&mut self, points: &[Point<Pixels>], colour: Hsla) {
        let mut path = PathBuilder::fill();
        path.add_polygon(points, true);
        if let Ok(path) = path.build() {
            self.window.paint_path(path, colour);
        }
    }

    /// Red above the pace, green below.
    fn colour(above: bool) -> Hsla {
        if above { p().danger } else { p().success }
    }

    fn now_line(&mut self) {
        let x = self.x(self.now);
        let (top, bottom) = (self.bounds.top(), self.bounds.bottom());
        self.line(&[point(x, top), point(x, bottom)], p().text.opacity(0.5), false, px(1.));
    }

    fn ratio(&mut self) {
        let Some(&(_, used)) = self.chart.points.last() else { return };
        let expected = self.steady(self.now);
        let b = self.bounds;
        let (top, bottom) = (b.top() + b.size.height * 0.2, b.bottom() - b.size.height * 0.2);
        let right = b.left() + b.size.width * (used / 100.) as f32;
        let colour = Pen::colour(used > expected);
        self.fill(&[point(b.left(), top), point(right, top), point(right, bottom), point(b.left(), bottom)], colour.opacity(0.7));
        let tick = b.left() + b.size.width * (expected / 100.) as f32;
        self.line(&[point(tick, b.top()), point(tick, b.bottom())], p().text, false, self.width * 1.5);
    }

    fn points(&mut self) {
        let gaps: Vec<(u64, f64)> = self.chart.points.iter().map(|(at, used)| (*at, used - self.steady(*at))).collect();
        let widest = gaps.iter().fold(WIDEST_GAP, |widest, (_, gap)| widest.max(gap.abs()));
        let y = |pen: &Pen, gap: f64| pen.y(0.5 + gap / widest / 2.);
        let zero = y(self, 0.);
        let (left, right) = (self.bounds.left(), self.bounds.right());
        self.line(&[point(left, zero), point(right, zero)], p().muted, true, px(1.));
        for pair in gaps.windows(2) {
            let ((t0, g0), (t1, g1)) = (pair[0], pair[1]);
            let (x0, x1) = (self.x(t0), self.x(t1));
            self.area(&[point(x0, zero), point(x0, y(self, g0)), point(x1, y(self, g1)), point(x1, zero)], Pen::colour(g0 + g1 > 0.));
        }
        let Some(&(_, last)) = gaps.last() else { return };
        let line: Vec<Point<Pixels>> = gaps.iter().map(|(at, gap)| point(self.x(*at), y(self, *gap))).collect();
        self.line(&line, Pen::colour(last > 0.), false, self.width);
        self.now_line();
    }

    fn wall(&mut self) {
        let left = |used: f64| (100. - used) / 100.;
        let points = self.chart.points.clone();
        for pair in points.windows(2) {
            let ((t0, u0), (t1, u1)) = (pair[0], pair[1]);
            let (s0, s1) = (self.steady(t0), self.steady(t1));
            let (x0, x1) = (self.x(t0), self.x(t1));
            let shape = [point(x0, self.y(left(u0))), point(x1, self.y(left(u1))), point(x1, self.y(left(s1))), point(x0, self.y(left(s0)))];
            self.area(&shape, Pen::colour(u0 + u1 > s0 + s1));
        }
        let pace = [point(self.x(self.chart.start), self.y(1.)), point(self.x(self.chart.resets_at), self.y(0.))];
        self.line(&pace, p().muted, true, px(1.));
        let Some(&(at, used)) = points.last() else { return };
        let colour = Pen::colour(used > self.steady(at));
        let line: Vec<Point<Pixels>> = points.iter().map(|(at, used)| point(self.x(*at), self.y(left(*used)))).collect();
        self.line(&line, colour, false, self.width);
        // On at the last hour's speed.
        if let Some(speeds) = self.chart.trend {
            let per_second = speeds.hour * 100. / self.length();
            let end = self.chart.wall.unwrap_or(self.chart.resets_at).max(at);
            let then = used + per_second * end.saturating_sub(at) as f64;
            let on = [point(self.x(at), self.y(left(used))), point(self.x(end), self.y(left(then)))];
            self.line(&on, colour, true, self.width);
        }
        if let Some(wall) = self.chart.wall {
            let x = self.x(wall);
            let (top, bottom) = (self.bounds.top(), self.bounds.bottom());
            self.line(&[point(x, top), point(x, bottom)], p().danger, false, self.width * 2.);
        }
        self.now_line();
    }

    fn trend(&mut self) {
        let along = crate::usage::trend_along(&self.chart.points, self.chart.window);
        let pace = self.y(1. / FASTEST);
        let (left, right) = (self.bounds.left(), self.bounds.right());
        self.line(&[point(left, pace), point(right, pace)], p().muted, true, px(1.));
        // The 15 minutes' speed, the older the fainter, as a scope's trace.
        let count = along.len();
        for (i, pair) in along.windows(2).enumerate() {
            let ((t0, s0), (t1, s1)) = (pair[0], pair[1]);
            let fade = 0.25 + 0.75 * ((i + 1) as f32 / count as f32).powf(1.5);
            let segment = [point(self.x(t0), self.y(s0.recent / FASTEST)), point(self.x(t1), self.y(s1.recent / FASTEST))];
            self.line(&segment, Pen::colour(s0.recent + s1.recent > 2.).opacity(fade), false, self.width);
        }
        if let Some(&(at, speeds)) = along.last() {
            let (x, y) = (self.x(at), self.y(speeds.recent / FASTEST));
            let dot = Bounds::new(point(x - self.width * 1.5, y - self.width * 1.5), size(self.width * 3., self.width * 3.));
            self.window.paint_quad(fill(dot, Pen::colour(speeds.recent > 1.)).corner_radii(self.width * 1.5));
        }
        self.now_line();
    }
}
