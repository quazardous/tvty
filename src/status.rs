//! A loop's Claude at a glance, from what aiball centralises: working or
//! idle (and for how long), who drives it — the loop on its own, held, or a
//! human typing — and whether the loop is connected at all.

use crate::ui::Named as _;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::sessions::Status;
use crate::theme::p;
use crate::tip::Tip as _;


impl Status {
    /// The state's colour: the accent beside the agent's name.
    pub fn colour(&self) -> Option<Hsla> {
        if !self.online {
            return None;
        }
        Some(match self.state.as_str() {
            "busy" => p().accent,
            // Yellow, as claude-loop's bar has always shown a boot.
            "boot" => p().warning,
            _ => p().muted,
        })
    }

    /// One small line: the driver's glyph, the state, for how long — and
    /// under the pointer, what they mean.
    /// `armed`: the loop's AFK mode, as its bar says it — held until let go
    /// (`wait_inf`) shows ■, as the folded list and the agent's bar do.
    pub fn line(&self, armed: Option<&str>) -> impl IntoElement + use<> {
        // Held until let go wins over a human typing: stop > pause.
        let for_good = matches!(self.driver.as_str(), "wait" | "stop") && armed == Some("wait_inf");
        if !self.online {
            return div()
                .named("status")
                .text_size(px(11.))
                .text_color(p().muted)
                .child("offline")
                .tip("its loop is not connected to aiball");
        }
        let who = match self.driver.as_str() {
            "stop" if for_good => "held until let go (a human is typing in it): the loop does not wake it",
            "stop" => "a human is typing in it: the loop waits",
            "wait" if for_good => "held until let go: the loop does not wake it",
            "wait" => "held a while: the loop does not wake it",
            "boot" => "starting",
            _ => "the loop drives it on its own",
        };
        let doing = match self.state.as_str() {
            "busy" => "Claude is working",
            "boot" => "Claude is starting",
            _ => "Claude is idle",
        };
        let (glyph, glyph_colour) = match self.driver.as_str() {
            _ if for_good => ("■", p().danger),
            "stop" => ("✎", p().danger),
            "wait" => ("‖", p().warning),
            "boot" => ("…", p().warning),
            _ => ("▶", p().success),
        };
        let what = match self.state.as_str() {
            "busy" => "working",
            "boot" => "starting",
            _ => "idle",
        };
        let since = self.since.map(|since| ago(now().saturating_sub(since)));
        div()
            .named("status")
            .flex()
            .items_center()
            .gap_1()
            .text_size(px(11.))
            .tip(format!("{doing}; {who}"))
            .child(crate::icons::loop_glyph(glyph, glyph_colour, 7.))
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

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `42s`, `3m`, `2h`, `5d`.
pub(crate) fn ago(seconds: u64) -> String {
    match seconds {
        0..60 => format!("{seconds}s"),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

/// When a step's agent resumes, short enough for a row: the local time
/// alone today, the day and time another day; none once it passed.
pub(crate) fn resume_short(iso: &str) -> Option<String> {
    resume_short_at(iso, chrono::Local::now())
}

fn resume_short_at<Tz: chrono::TimeZone>(iso: &str, now: chrono::DateTime<Tz>) -> Option<String>
where
    Tz::Offset: std::fmt::Display,
{
    let at = chrono::DateTime::parse_from_rfc3339(iso).ok()?.with_timezone(&now.timezone());
    if at <= now {
        return None;
    }
    let pattern = if at.date_naive() == now.date_naive() { "%H:%M" } else { "%d/%m %H:%M" };
    Some(at.format(pattern).to_string())
}

/// The same, whole: `2026-09-28 00:33`, local.
pub(crate) fn resume_full(iso: &str) -> Option<String> {
    let at = chrono::DateTime::parse_from_rfc3339(iso).ok()?.with_timezone(&chrono::Local);
    Some(at.format("%Y-%m-%d %H:%M").to_string())
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

/// An ISO-8601 UTC time, as aiball reads it.
pub fn format_time(at: std::time::SystemTime) -> String {
    let secs = at.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rest) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // The civil date from days (Howard Hinnant's algorithm).
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    )
}

#[cfg(test)]
mod resume_tests {
    use super::resume_short_at;
    use chrono::TimeZone as _;

    #[test]
    fn a_resume_says_its_time_today_its_day_else_nothing_once_passed() {
        let paris = chrono::FixedOffset::east_opt(2 * 3600).unwrap();
        let now = paris.with_ymd_and_hms(2026, 9, 28, 0, 10, 0).unwrap();
        // 22:33 UTC is 00:33 in Paris, the same day.
        assert_eq!(resume_short_at("2026-09-27T22:33:00.000Z", now).as_deref(), Some("00:33"));
        assert_eq!(resume_short_at("2026-09-28T22:33:00.000Z", now).as_deref(), Some("29/09 00:33"));
        assert_eq!(resume_short_at("2026-09-27T22:00:00.000Z", now), None);
        assert_eq!(resume_short_at("not a time", now), None);
    }
}

#[cfg(test)]
mod tests {
    use super::{ago, format_time, parse_time};

    #[test]
    fn formats_what_it_parses() {
        let at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_790_264_157);
        assert_eq!(format_time(at), "2026-09-24T15:35:57Z");
        assert_eq!(parse_time(&format_time(at)), Some(1_790_264_157));
    }

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
