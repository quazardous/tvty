//! How fast the subscription goes, against the pace that would use it up
//! right at its window's end: what the arrow in the top bar says. The usage
//! is Claude Code's own (its status line's `rate_limits`), which a loop
//! pushes to aiball in its bar; the pace is worked out here. The readings
//! are kept ([`History`]): the curve in the top bar and the recent speed
//! (the trend, the wall) are drawn from them.

use serde::{Deserialize, Serialize};

use crate::aiball::BarRead;
use crate::config::{self, Place, Stored};
use crate::status::parse_time;

/// The subscription's usage as a loop read it from its Claude.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarUsage {
    #[serde(default)]
    pub five_hour: Option<UsageWindow>,
    #[serde(default)]
    pub seven_day: Option<UsageWindow>,
    pub read_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct UsageWindow {
    /// 0 to 100.
    pub used_percentage: f64,
    pub resets_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    FiveHour,
    SevenDay,
}

impl Window {
    pub fn length(self) -> u64 {
        match self {
            Window::FiveHour => 5 * 3600,
            Window::SevenDay => 7 * 86400,
        }
    }

    /// `usage-window-five_hour`.
    pub fn said(self) -> String {
        crate::t!(match self {
            Window::FiveHour => "usage-window-five_hour",
            Window::SevenDay => "usage-window-seven_day",
        })
    }
}

/// The gap, in points of the quota, the arrow is at its reddest (or
/// greenest) from.
pub const FULL_AT: f64 = 20.;

/// A window's usage against the steady pace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pace {
    pub window: Window,
    /// Percent of the quota used.
    pub used: f64,
    /// Percent a steady pace would have used by now.
    pub expected: f64,
    /// Seconds since the epoch.
    pub resets_at: u64,
    elapsed: u64,
}

impl Pace {
    /// None once the window reset: its reading is of the one before.
    pub fn of(window: Window, reading: &UsageWindow, now: u64) -> Option<Pace> {
        let resets_at = parse_time(&reading.resets_at)?;
        if resets_at <= now {
            return None;
        }
        let length = window.length();
        let elapsed = length.saturating_sub(resets_at - now);
        Some(Pace {
            window,
            used: reading.used_percentage.clamp(0., 100.),
            expected: elapsed as f64 / length as f64 * 100.,
            resets_at,
            elapsed,
        })
    }

    /// Above the pace (positive) or below, in points of the quota.
    pub fn points(&self) -> f64 {
        self.used - self.expected
    }

    /// Used against expected; none at the window's very start.
    pub fn ratio(&self) -> Option<f64> {
        (self.expected > 0.).then(|| self.used / self.expected)
    }

    /// Seconds until the quota is used up at the pace so far; none when
    /// the window resets first.
    pub fn wall(&self, now: u64) -> Option<u64> {
        if self.used <= 0. || self.elapsed == 0 {
            return None;
        }
        let per_second = self.used / self.elapsed as f64;
        let left = ((100. - self.used) / per_second) as u64;
        (now + left < self.resets_at).then_some(left)
    }

    /// -1 (well below the pace) to 1 (well above): how green or red.
    pub fn heat(&self) -> f64 {
        (self.points() / FULL_AT).clamp(-1., 1.)
    }

    /// Above the pace.
    pub fn above(&self) -> bool {
        self.points() > 0.
    }
}

/// The window the arrow says: the five hours, else the week.
pub fn pace(usage: &BarUsage, now: u64) -> Option<Pace> {
    usage
        .five_hour
        .as_ref()
        .and_then(|w| Pace::of(Window::FiveHour, w, now))
        .or_else(|| usage.seven_day.as_ref().and_then(|w| Pace::of(Window::SevenDay, w, now)))
}

/// Every window still running, for the tip.
pub fn paces(usage: &BarUsage, now: u64) -> Vec<Pace> {
    [(Window::FiveHour, &usage.five_hour), (Window::SevenDay, &usage.seven_day)]
        .into_iter()
        .filter_map(|(window, reading)| Pace::of(window, reading.as_ref()?, now))
        .collect()
}

/// The newest reading among the loops' bars: the quota is the account's,
/// so any loop's reading is everyone's. A gone loop's last one counts too.
pub fn freshest<'a>(bars: impl IntoIterator<Item = &'a BarRead>) -> Option<&'a BarUsage> {
    bars.into_iter()
        .filter_map(|read| read.bar.usage.as_ref())
        .filter_map(|usage| Some((parse_time(&usage.read_at)?, usage)))
        .max_by_key(|(at, _)| *at)
        .map(|(_, usage)| usage)
}

/// How the arrow's figure says the gap; a click on it goes to the next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    /// `×1.3`: used against expected.
    #[default]
    Ratio,
    /// `+12 pts`.
    Points,
    /// `wall 1h40`: when the quota runs out at the recent speed.
    Wall,
    /// `speed ×1.8`: the last 15 minutes' speed, in multiples of the
    /// steady pace.
    Trend,
}

impl Display {
    pub const ALL: [Display; 4] = [Display::Ratio, Display::Points, Display::Wall, Display::Trend];

    /// Its word in `settings.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Display::Ratio => "ratio",
            Display::Points => "points",
            Display::Wall => "wall",
            Display::Trend => "trend",
        }
    }

    pub fn from_key(key: &str) -> Option<Display> {
        Display::ALL.into_iter().find(|d| d.key() == key)
    }

    pub fn next(self) -> Display {
        let at = Display::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Display::ALL[(at + 1) % Display::ALL.len()]
    }
}

/// The figure beside the arrow; `chart` gives the recent speed, else the
/// wall is the average pace's.
pub fn said(pace: &Pace, chart: Option<&Chart>, display: Display, now: u64) -> String {
    match display {
        Display::Ratio => match pace.ratio() {
            Some(ratio) if ratio < 10. => crate::t!("usage-ratio", ratio = format!("{ratio:.1}")),
            Some(_) => crate::t!("usage-ratio", ratio = ">9.9"),
            None => crate::t!("usage-ratio", ratio = "—"),
        },
        Display::Points => {
            let points = pace.points().round();
            let sign = if points > 0. { "+" } else if points < 0. { "−" } else { "" };
            crate::t!("usage-points", points = format!("{sign}{}", points.abs()))
        }
        Display::Wall => match wall_in(pace, chart, now) {
            Some(left) => crate::t!("usage-wall", left = span(left)),
            None => crate::t!("usage-no-wall"),
        },
        Display::Trend => crate::t!("usage-trend", speed = chart.and_then(|c| c.trend).map_or("—".into(), |s| times(s.recent))),
    }
}

/// Seconds until the wall: at the recent speed when it is known.
fn wall_in(pace: &Pace, chart: Option<&Chart>, now: u64) -> Option<u64> {
    match chart.filter(|c| c.trend.is_some()) {
        Some(chart) => chart.wall.map(|at| at.saturating_sub(now)),
        None => pace.wall(now),
    }
}

/// `1.8`, `>9.9`.
fn times(speed: f64) -> String {
    if speed < 10. { format!("{speed:.1}") } else { ">9.9".into() }
}

/// `45m`, `1h40`, `2d 4h`.
pub fn span(seconds: u64) -> String {
    let minutes = seconds / 60;
    match minutes {
        0..60 => format!("{minutes}m"),
        60..1440 => format!("{}h{:02}", minutes / 60, minutes % 60),
        _ => format!("{}d {}h", minutes / 1440, minutes % 1440 / 60),
    }
}

/// What the arrow's tip says: each window's usage and pace, and where it
/// goes at this pace.
pub fn tip(usage: &BarUsage, chart: Option<&Chart>, now: u64) -> String {
    let mut lines: Vec<String> = paces(usage, now)
        .iter()
        .map(|pace| {
            let resets = chrono::DateTime::from_timestamp(pace.resets_at as i64, 0)
                .map(|at| crate::status::resume_short(&at.to_rfc3339()).unwrap_or_default())
                .unwrap_or_default();
            let shown = chart.filter(|c| c.window == pace.window);
            let end = match wall_in(pace, shown, now) {
                Some(left) => crate::t!("usage-tip-wall", left = span(left)),
                None => crate::t!("usage-tip-lasts"),
            };
            crate::t!(
                "usage-tip-window",
                window = pace.window.said(),
                used = format!("{:.0}", pace.used),
                expected = format!("{:.0}", pace.expected),
                resets = resets,
                end = end
            )
        })
        .collect();
    if let Some(speeds) = chart.and_then(|c| c.trend) {
        lines.push(crate::t!("usage-tip-trend", recent = times(speeds.recent), hour = times(speeds.hour)));
    }
    if let Some(at) = parse_time(&usage.read_at) {
        lines.push(crate::t!("usage-tip-read", ago = span(now.saturating_sub(at))));
    }
    lines.push(crate::t!("usage-tip-click"));
    lines.join("\n")
}

// ── The readings kept: the curve, the recent speed ───────────────────────

/// A window's reading, kept.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub window: Window,
    /// Seconds since the epoch; tells the window apart from the next one.
    pub resets_at: u64,
    /// When it was read.
    pub at: u64,
    pub used: f64,
}

/// `usage.json`: the readings of the windows still running, kept by tvty
/// on its own, so that a restart keeps the curve.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    pub samples: Vec<Sample>,
}

impl Stored for History {
    const PLACE: Place = Place::State;
    const FILE: &'static str = "usage.json";
}

pub fn init(cx: &mut gpui_kit::App) {
    config::register::<History>(cx);
}

/// A reading saying the same as the last one is kept only this long after
/// it: the curve needs a point now and then, not one a minute.
const KEEP_SAME: u64 = 10 * 60;

/// Two readings of one window: its end may be said a little apart.
fn same_window(a: u64, b: u64) -> bool {
    a.abs_diff(b) < 10 * 60
}

impl History {
    /// Takes `usage` in, when it says something new; forgets the windows
    /// gone by.
    pub fn take(&mut self, usage: &BarUsage, now: u64) {
        if let Some(at) = parse_time(&usage.read_at) {
            for (window, reading) in [(Window::FiveHour, &usage.five_hour), (Window::SevenDay, &usage.seven_day)] {
                let Some(reading) = reading else { continue };
                let Some(resets_at) = parse_time(&reading.resets_at) else { continue };
                let last = self.samples.iter().rev().find(|s| s.window == window && same_window(s.resets_at, resets_at));
                if last.is_some_and(|l| at <= l.at || (l.used == reading.used_percentage && at - l.at < KEEP_SAME)) {
                    continue;
                }
                self.samples.push(Sample { window, resets_at, at, used: reading.used_percentage.clamp(0., 100.) });
            }
        }
        self.samples.retain(|s| s.resets_at > now);
    }

    /// The window's curve: from its start, where nothing is used, through
    /// each reading, in time.
    pub fn points(&self, window: Window, resets_at: u64) -> Vec<(u64, f64)> {
        let start = resets_at.saturating_sub(window.length());
        let mut points: Vec<(u64, f64)> =
            self.samples.iter().filter(|s| s.window == window && same_window(s.resets_at, resets_at) && s.at >= start).map(|s| (s.at, s.used)).collect();
        points.sort_by_key(|(at, _)| *at);
        points.insert(0, (start, 0.));
        points
    }
}

/// The recent speed, in multiples of the steady pace (1: on it): averages
/// that forget at an exponential rate, as a load average does, which takes
/// readings at any interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speeds {
    /// Over about 15 minutes: what the trace and the figure say.
    pub recent: f64,
    /// Over about an hour: what places the wall, a short burst aside.
    pub hour: f64,
}

const RECENT: f64 = 15. * 60.;
const HOUR: f64 = 3600.;

/// The speeds after the last reading; none without two points.
pub fn trend(points: &[(u64, f64)], window: Window) -> Option<Speeds> {
    trend_along(points, window).last().map(|(_, speeds)| *speeds)
}

/// [`trend`] after each reading: the speed's trace, in time.
pub fn trend_along(points: &[(u64, f64)], window: Window) -> Vec<(u64, Speeds)> {
    let pace = 100. / window.length() as f64;
    let mut along = Vec::new();
    let mut speeds: Option<Speeds> = None;
    for pair in points.windows(2) {
        let ((t0, u0), (t1, u1)) = (pair[0], pair[1]);
        let dt = t1.saturating_sub(t0);
        if dt == 0 {
            continue;
        }
        let speed = (u1 - u0).max(0.) / dt as f64 / pace;
        let forget = |period: f64| 1. - (-(dt as f64) / period).exp();
        speeds = Some(match speeds {
            None => Speeds { recent: speed, hour: speed },
            Some(kept) => Speeds { recent: kept.recent + forget(RECENT) * (speed - kept.recent), hour: kept.hour + forget(HOUR) * (speed - kept.hour) },
        });
        along.extend(speeds.map(|speeds| (t1, speeds)));
    }
    along
}

/// What the curve in the top bar draws: one window, from its start to its
/// reset.
#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub window: Window,
    pub start: u64,
    pub resets_at: u64,
    /// (when, percent used), from the start.
    pub points: Vec<(u64, f64)>,
    pub trend: Option<Speeds>,
    /// When the quota runs out at the last hour's speed; none when the
    /// window resets first.
    pub wall: Option<u64>,
}

/// The chart of the window [`pace`] says, from what is kept and the
/// reading in hand (it may not be kept yet).
pub fn chart(history: &History, usage: &BarUsage, now: u64) -> Option<Chart> {
    let pace = pace(usage, now)?;
    let (window, resets_at) = (pace.window, pace.resets_at);
    let mut points = history.points(window, resets_at);
    if let Some(at) = parse_time(&usage.read_at).filter(|at| points.last().is_some_and(|(last, _)| at > last)) {
        points.push((at, pace.used));
    }
    let trend = trend(&points, window);
    let wall = trend.and_then(|speeds| {
        let (at, used) = *points.last()?;
        let per_second = speeds.hour * 100. / window.length() as f64;
        (per_second > 0.).then(|| at + ((100. - used) / per_second) as u64).filter(|wall| *wall < resets_at)
    });
    Some(Chart { window, start: resets_at.saturating_sub(window.length()), resets_at, points, trend, wall })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_790_000_000;

    fn iso(at: u64) -> String {
        chrono::DateTime::from_timestamp(at as i64, 0).unwrap().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    fn window(used: f64, resets_in: u64) -> UsageWindow {
        UsageWindow { used_percentage: used, resets_at: iso(NOW + resets_in) }
    }

    #[test]
    fn the_pace_is_the_share_of_the_window_gone_by() {
        // Two hours into five, with half the quota used: above the pace.
        let pace = Pace::of(Window::FiveHour, &window(50., 3 * 3600), NOW).unwrap();
        assert!((pace.expected - 40.).abs() < 1e-9, "{pace:?}");
        assert!((pace.points() - 10.).abs() < 1e-9);
        assert!((pace.ratio().unwrap() - 1.25).abs() < 1e-9);
        assert!(pace.above());
        assert!((pace.heat() - 0.5).abs() < 1e-9);
        // 25 points an hour: the other half goes in two hours, before the reset.
        assert_eq!(pace.wall(NOW), Some(2 * 3600));
        // Below it: no wall, the window resets first.
        let calm = Pace::of(Window::FiveHour, &window(20., 3 * 3600), NOW).unwrap();
        assert!(!calm.above());
        assert_eq!(calm.wall(NOW), None);
        assert!((calm.heat() + 1.).abs() < 1e-9);
    }

    #[test]
    fn a_window_gone_by_says_nothing() {
        assert_eq!(Pace::of(Window::FiveHour, &UsageWindow { used_percentage: 90., resets_at: iso(NOW - 1) }, NOW), None);
        // The week then.
        let usage = BarUsage { five_hour: Some(UsageWindow { used_percentage: 90., resets_at: iso(NOW - 1) }), seven_day: Some(window(10., 86400)), read_at: iso(NOW) };
        assert_eq!(pace(&usage, NOW).map(|p| p.window), Some(Window::SevenDay));
    }

    #[test]
    fn the_figure_follows_the_display() {
        let pace = Pace::of(Window::FiveHour, &window(50., 3 * 3600), NOW).unwrap();
        assert_eq!(said(&pace, None, Display::Ratio, NOW), "×1.2");
        assert_eq!(said(&pace, None, Display::Points, NOW), "+10 pts");
        assert_eq!(said(&pace, None, Display::Wall, NOW), "wall 2h00");
        let calm = Pace::of(Window::FiveHour, &window(20., 3 * 3600), NOW).unwrap();
        assert_eq!(said(&calm, None, Display::Points, NOW), "−20 pts");
        assert_eq!(said(&calm, None, Display::Wall, NOW), "no wall");
    }

    #[test]
    fn a_click_goes_round_the_displays() {
        assert_eq!(Display::Ratio.next(), Display::Points);
        assert_eq!(Display::Points.next(), Display::Wall);
        assert_eq!(Display::Wall.next(), Display::Trend);
        assert_eq!(Display::Trend.next(), Display::Ratio);
        assert_eq!(Display::from_key("wall"), Some(Display::Wall));
        assert_eq!(Display::from_key("nope"), None);
    }

    fn reading(five_hour: Option<UsageWindow>, at: u64) -> BarUsage {
        BarUsage { five_hour, seven_day: None, read_at: iso(at) }
    }

    #[test]
    fn readings_are_kept_while_their_window_runs() {
        let mut history = History::default();
        let resets = NOW + 3 * 3600;
        history.take(&reading(Some(UsageWindow { used_percentage: 30., resets_at: iso(resets) }), NOW), NOW);
        // The same reading again, or the same figure soon after: nothing new.
        history.take(&reading(Some(UsageWindow { used_percentage: 30., resets_at: iso(resets) }), NOW), NOW);
        history.take(&reading(Some(UsageWindow { used_percentage: 30., resets_at: iso(resets) }), NOW + 60), NOW + 60);
        assert_eq!(history.samples.len(), 1);
        history.take(&reading(Some(UsageWindow { used_percentage: 34., resets_at: iso(resets) }), NOW + 600), NOW + 600);
        // From the window's start, where nothing was used.
        assert_eq!(history.points(Window::FiveHour, resets), vec![(resets - 5 * 3600, 0.), (NOW, 30.), (NOW + 600, 34.)]);
        // Once it reset, forgotten.
        history.take(&reading(None, resets + 1), resets + 1);
        assert!(history.samples.is_empty());
    }

    #[test]
    fn the_trend_follows_the_recent_speed() {
        // Two hours at the pace (40 %), then twice as fast for an hour.
        let start = NOW;
        let mut points = vec![(start, 0.)];
        for minute in (10..=120).step_by(10) {
            points.push((start + minute * 60, minute as f64 / 3.));
        }
        for minute in (130..=180).step_by(10) {
            points.push((start + minute * 60, 40. + (minute - 120) as f64 * 2. / 3.));
        }
        let Speeds { recent, hour } = trend(&points, Window::FiveHour).unwrap();
        assert!(recent > 1.9 && recent <= 2., "{recent}");
        assert!(hour > 1.5 && hour < recent, "{hour}");
        // Nothing but the start: no trend.
        assert_eq!(trend(&points[..1], Window::FiveHour), None);
    }

    #[test]
    fn the_wall_comes_at_the_last_hours_speed() {
        // Two hours in, 40 % used: on the pace, then 40 points in the last
        // hour, twice the pace: 20 % left, gone well before the reset.
        let resets = NOW + 2 * 3600;
        let mut history = History::default();
        for (minutes_ago, used) in [(60, 40.), (30, 60.)] {
            history.take(&reading(Some(UsageWindow { used_percentage: used, resets_at: iso(resets) }), NOW - minutes_ago * 60), NOW);
        }
        let chart = chart(&history, &reading(Some(UsageWindow { used_percentage: 80., resets_at: iso(resets) }), NOW), NOW).unwrap();
        assert_eq!(chart.points.len(), 4);
        let wall = chart.wall.expect("a wall before the reset");
        assert!(wall > NOW && wall < resets, "{}", wall as i64 - NOW as i64);
    }

    #[test]
    fn spans() {
        assert_eq!(span(45 * 60), "45m");
        assert_eq!(span(100 * 60), "1h40");
        assert_eq!(span(52 * 3600), "2d 4h");
    }
}
