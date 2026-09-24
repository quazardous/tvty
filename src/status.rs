//! A loop's Claude at a glance, from what aiball centralises: working or
//! idle (and for how long), who drives it — the loop on its own, held, or a
//! human typing — and whether the loop is connected at all.

use std::time::{SystemTime, UNIX_EPOCH};

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::sessions::Status;
use crate::theme::p;


impl Status {
    /// The state's colour: the accent beside the agent's name.
    pub fn colour(&self) -> Option<Hsla> {
        if !self.online {
            return None;
        }
        Some(match self.state.as_str() {
            "busy" => p().accent,
            "boot" => p().info,
            _ => p().muted,
        })
    }

    /// One small line: the driver's glyph, the state, for how long.
    pub fn line(&self) -> impl IntoElement + use<> {
        if !self.online {
            return div()
                .text_size(px(11.))
                .text_color(p().muted)
                .child("offline");
        }
        let (glyph, glyph_colour) = match self.driver.as_str() {
            "stop" => ("✎", p().danger),
            "wait" => ("‖", p().warning),
            "boot" => ("…", p().info),
            _ => ("▶", p().success),
        };
        let what = match self.state.as_str() {
            "busy" => "working",
            "boot" => "starting",
            _ => "idle",
        };
        let since = self.since.map(|since| ago(now().saturating_sub(since)));
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_size(px(11.))
            .child(div().text_color(glyph_colour).child(glyph))
            .child(
                div()
                    .text_color(match self.state.as_str() {
                        "busy" => p().accent,
                        _ => p().muted,
                    })
                    .child(what),
            )
            .when_some(since, |d, since| {
                d.child(div().text_color(p().muted).child(format!("· {since}")))
            })
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `42s`, `3m`, `2h`, `5d`.
fn ago(seconds: u64) -> String {
    match seconds {
        0..60 => format!("{seconds}s"),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

/// Seconds since the epoch of an ISO-8601 UTC time as aiball writes it
/// (`2026-09-24T15:35:57.678Z`).
pub fn parse_time(text: &str) -> Option<u64> {
    let date = text.get(..10)?;
    let time = text.get(11..19)?;
    let mut d = date.split('-').map(|p| p.parse::<i64>());
    let (y, m, day) = (d.next()?.ok()?, d.next()?.ok()?, d.next()?.ok()?);
    let mut t = time.split(':').map(|p| p.parse::<i64>());
    let (h, min, s) = (t.next()?.ok()?, t.next()?.ok()?, t.next()?.ok()?);
    // Days from the civil date (Howard Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    u64::try_from(days * 86400 + h * 3600 + min * 60 + s).ok()
}

#[cfg(test)]
mod tests {
    use super::{ago, parse_time};

    #[test]
    fn parses_aiball_times() {
        assert_eq!(parse_time("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(parse_time("2026-09-24T15:35:57.678Z"), Some(1_790_264_157));
        assert_eq!(parse_time("garbage"), None);
    }

    #[test]
    fn says_how_long_ago() {
        assert_eq!(ago(42), "42s");
        assert_eq!(ago(180), "3m");
        assert_eq!(ago(7200), "2h");
        assert_eq!(ago(3 * 86400), "3d");
    }
}
