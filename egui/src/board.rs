//! A ticket row as aiball gives it (`inbox.list`, `view=turn`), and what
//! the GPUI build reads from it at a glance: its band, whose turn, its
//! glyph and the stripe on its left (tvty's `rowstate`), the badges of
//! what a set of tickets asks of the user (its `Alerts`).

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Ticket {
    pub id: u64,
    #[serde(default)]
    pub project: String,
    pub title: String,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub by_agent: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub closed: bool,
    pub intent: Option<String>,
    pub priority: Option<String>,
    pub level: Option<String>,
    pub scope: Option<String>,
    pub critical: Option<Critical>,
    #[serde(default)]
    pub unread: bool,
    #[serde(default)]
    pub hot: bool,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub has_payload: bool,
    pub postponed_until: Option<String>,
    #[serde(default)]
    pub comment_count: u32,
    #[serde(default)]
    pub pending_comment_count: u32,
    pub last_activity: Option<String>,
    pub last_speaker: Option<String>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    pub holder: Option<String>,
    pub held_as: Option<String>,
    #[serde(default)]
    pub pending_plan: bool,
    #[serde(default)]
    pub pending_resolution: bool,
    #[serde(default)]
    pub pending_wontfix: bool,
    #[serde(default)]
    pub pending_escalation: bool,
    #[serde(default)]
    pub pending_decision_is_latest: bool,
    pub turn: Option<String>,
    pub band_name: Option<String>,
    pub state_glyph: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Critical {
    #[serde(default)]
    pub holds: u32,
    pub quiet_since: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Tag {
    pub name: String,
}

impl Critical {
    /// How long nothing moved on it ("9d"), once a day went by.
    pub fn quiet(&self) -> Option<String> {
        let seconds = now().saturating_sub(parse_time(self.quiet_since.as_deref()?)?);
        (seconds >= 86_400).then(|| format!("{}d", seconds / 86_400))
    }
}

impl Ticket {
    pub fn pending_decision(&self) -> bool {
        self.pending_plan || self.pending_resolution || self.pending_wontfix || self.pending_escalation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Moderate,
    Decide,
    AgentOnIt,
    Open,
    Closed,
}

impl Band {
    /// The full list's order.
    pub const ALL: [Band; 5] = [Band::AgentOnIt, Band::Moderate, Band::Decide, Band::Open, Band::Closed];

    fn from_server(name: &str) -> Option<Band> {
        Some(match name {
            "moderate" => Band::Moderate,
            "decision" => Band::Decide,
            "working" => Band::AgentOnIt,
            "open" => Band::Open,
            "closed" => Band::Closed,
            _ => return None,
        })
    }

    pub fn title(self) -> &'static str {
        match self {
            Band::Moderate => "To moderate",
            Band::Decide => "Waiting on you",
            Band::AgentOnIt => "Agents on it",
            Band::Open => "Open",
            Band::Closed => "Closed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Turn {
    You,
    Them,
    Nobody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Escalation,
    Plan,
    Resolution,
    Wontfix,
    StalledStep,
    Step,
    Rejected,
    ClosedResolved,
    Closed,
}

impl Glyph {
    fn from_server(name: &str) -> Option<Glyph> {
        Some(match name {
            "escalation" => Glyph::Escalation,
            "plan" => Glyph::Plan,
            "resolution" => Glyph::Resolution,
            "wontfix" => Glyph::Wontfix,
            "step_stalled" => Glyph::StalledStep,
            "step" => Glyph::Step,
            "rejected" => Glyph::Rejected,
            "closed_resolved" => Glyph::ClosedResolved,
            "closed" => Glyph::Closed,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stripe {
    Solid,
    Dashed,
    Neutral,
    Waiting,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowState {
    pub band: Band,
    pub turn: Turn,
    pub glyph: Option<Glyph>,
    pub stripe: Stripe,
}

/// As tvty reads a row for `user`.
pub fn state_of(row: &Ticket, user: &str) -> RowState {
    let band = row.band_name.as_deref().and_then(Band::from_server).unwrap_or(Band::Open);
    let turn = match row.turn.as_deref() {
        Some("you") => Turn::You,
        Some("them") => Turn::Them,
        _ => Turn::Nobody,
    };
    let glyph = row.state_glyph.as_deref().and_then(Glyph::from_server);
    let mut stripe = match turn {
        Turn::You if row.pending_decision() && row.pending_decision_is_latest => Stripe::Solid,
        Turn::You if row.pending_decision() => Stripe::Dashed,
        Turn::You => Stripe::Neutral,
        _ => Stripe::None,
    };
    if stripe == Stripe::None && row.last_speaker.as_deref() == Some(user) {
        stripe = Stripe::Waiting;
    }
    RowState { band, turn, glyph, stripe }
}

/// What a set of tickets asks of the user: the critical ticket among
/// them, the decisions waiting, what is unread.
#[derive(Clone, Copy, Debug, Default)]
pub struct Alerts {
    pub critical: bool,
    pub decisions: usize,
    pub unread: usize,
}

impl Alerts {
    pub fn of<'a>(tickets: impl Iterator<Item = &'a Ticket>) -> Self {
        let mut alerts = Alerts::default();
        for t in tickets {
            alerts.critical |= t.critical.is_some();
            alerts.decisions += t.pending_decision() as usize;
            alerts.unread += t.unread as usize;
        }
        alerts
    }
}

/// Seconds since the epoch of `2026-09-24T13:01:55.681Z`.
pub fn parse_time(when: &str) -> Option<u64> {
    let n = |range: std::ops::Range<usize>| when.get(range)?.parse::<i64>().ok();
    let (y, m, d) = (n(0..4)?, n(5..7)?, n(8..10)?);
    let (hh, mm, ss) = (n(11..13)?, n(14..16)?, n(17..19)?);
    // Days from the civil date (Howard Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    u64::try_from(days * 86_400 + hh * 3600 + mm * 60 + ss).ok()
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// `3m`, `2h`, `5d` ago.
pub fn ago(when: &str) -> Option<String> {
    let seconds = now().saturating_sub(parse_time(when)?);
    Some(match seconds {
        0..60 => "now".to_string(),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_read_as_the_epoch_counts_them() {
        assert_eq!(parse_time("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(parse_time("2026-10-22T14:30:30.336Z"), Some(1_792_679_430));
    }

    #[test]
    fn a_decision_waiting_and_last_is_a_solid_stripe() {
        let row = Ticket {
            pending_plan: true,
            pending_decision_is_latest: true,
            turn: Some("you".into()),
            band_name: Some("decision".into()),
            state_glyph: Some("plan".into()),
            ..Default::default()
        };
        let s = state_of(&row, "david");
        assert_eq!((s.band, s.turn, s.glyph, s.stripe), (Band::Decide, Turn::You, Some(Glyph::Plan), Stripe::Solid));
    }

    #[test]
    fn your_word_last_is_a_waiting_stripe() {
        let row = Ticket { turn: Some("them".into()), last_speaker: Some("david".into()), ..Default::default() };
        assert_eq!(state_of(&row, "david").stripe, Stripe::Waiting);
    }
}
