//! How fast the subscription goes, against the pace that would use it up
//! right at its window's end: what the arrow in the top bar says. The usage
//! is Claude Code's own (its status line's `rate_limits`), which a loop
//! pushes to aiball in its bar; the pace is worked out here.

use serde::Deserialize;

use crate::aiball::BarRead;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    FiveHour,
    SevenDay,
}

impl Window {
    fn length(self) -> u64 {
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
    /// `wall 1h40`: when the quota runs out at this pace.
    Wall,
}

impl Display {
    pub const ALL: [Display; 3] = [Display::Ratio, Display::Points, Display::Wall];

    /// Its word in `settings.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Display::Ratio => "ratio",
            Display::Points => "points",
            Display::Wall => "wall",
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

/// The figure beside the arrow.
pub fn said(pace: &Pace, display: Display, now: u64) -> String {
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
        Display::Wall => match pace.wall(now) {
            Some(left) => crate::t!("usage-wall", left = span(left)),
            None => crate::t!("usage-no-wall"),
        },
    }
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
pub fn tip(usage: &BarUsage, now: u64) -> String {
    let mut lines: Vec<String> = paces(usage, now)
        .iter()
        .map(|pace| {
            let resets = chrono::DateTime::from_timestamp(pace.resets_at as i64, 0)
                .map(|at| crate::status::resume_short(&at.to_rfc3339()).unwrap_or_default())
                .unwrap_or_default();
            let end = match pace.wall(now) {
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
    lines.push(crate::t!("usage-tip-click"));
    lines.join("\n")
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
        assert_eq!(said(&pace, Display::Ratio, NOW), "×1.2");
        assert_eq!(said(&pace, Display::Points, NOW), "+10 pts");
        assert_eq!(said(&pace, Display::Wall, NOW), "wall 2h00");
        let calm = Pace::of(Window::FiveHour, &window(20., 3 * 3600), NOW).unwrap();
        assert_eq!(said(&calm, Display::Points, NOW), "−20 pts");
        assert_eq!(said(&calm, Display::Wall, NOW), "no wall");
    }

    #[test]
    fn a_click_goes_round_the_displays() {
        assert_eq!(Display::Ratio.next(), Display::Points);
        assert_eq!(Display::Points.next(), Display::Wall);
        assert_eq!(Display::Wall.next(), Display::Ratio);
        assert_eq!(Display::from_key("wall"), Some(Display::Wall));
        assert_eq!(Display::from_key("nope"), None);
    }

    #[test]
    fn spans() {
        assert_eq!(span(45 * 60), "45m");
        assert_eq!(span(100 * 60), "1h40");
        assert_eq!(span(52 * 3600), "2d 4h");
    }
}
