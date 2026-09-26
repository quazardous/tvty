//! What a ticket row tells at a glance: its band in the list, whose turn it
//! is, one state glyph, and the stripe on its left. aiball computes the band,
//! the turn and the glyph (`/api/inbox?v=tvty`); tvty draws them, and derives
//! only the stripe (see docs/UX.md, "The ticket list").

use crate::aiball::TicketRow;

/// Where the row sorts: from "it is yours" to "it runs by itself".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Band {
    /// The ticket, or comments on it, wait for moderation.
    Moderate,
    /// A plan, resolution, wontfix or escalation waits for your decision.
    Decide,
    /// Something new on it.
    Unread,
    /// An agent holds it or is on a step.
    AgentOnIt,
    /// Open, nothing pressing.
    Open,
    Closed,
}

impl Band {
    /// The order the list shows the bands in: what agents are on first —
    /// the work under way —, then what waits on you, then the rest.
    pub fn rank(self) -> u8 {
        match self {
            Band::AgentOnIt => 0,
            Band::Moderate => 1,
            Band::Decide => 2,
            Band::Unread => 3,
            Band::Open => 4,
            Band::Closed => 5,
        }
    }

    /// aiball's band number, 0 to 5, in this order.
    fn from_server(band: u8) -> Option<Band> {
        [Band::Moderate, Band::Decide, Band::Unread, Band::AgentOnIt, Band::Open, Band::Closed]
            .get(band as usize)
            .copied()
    }

    pub fn title(self) -> &'static str {
        match self {
            Band::Moderate => "To moderate",
            Band::Decide => "Waiting on you",
            Band::Unread => "Unread",
            Band::AgentOnIt => "Agents on it",
            Band::Open => "Open",
            Band::Closed => "Closed",
        }
    }
}

/// Who has to move next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Turn {
    /// You: moderate, decide, or answer the agent who spoke last.
    You,
    /// An agent: you spoke last, or it is on a step.
    Them,
    /// Nobody: closed, or nothing said yet.
    Nobody,
}

/// The one state glyph of a row; `None` when there is nothing special.
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
    /// aiball's `state_glyph` name.
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

    pub fn meaning(self) -> &'static str {
        match self {
            Glyph::Escalation => "an agent escalates: it needs you to act",
            Glyph::Plan => "a plan is proposed",
            Glyph::Resolution => "a resolution is proposed",
            Glyph::Wontfix => "closing without a fix is proposed",
            Glyph::StalledStep => "an agent's step went quiet",
            Glyph::Step => "an agent is on a step (then: continue)",
            Glyph::Rejected => "the last plan or resolution was rejected",
            Glyph::ClosedResolved => "closed, resolved",
            Glyph::Closed => "closed without a resolution",
        }
    }
}

/// The stripe on the row's left: whose turn, and how fresh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stripe {
    /// A decision waits on you and is the last message.
    Solid,
    /// A decision waits on you, but the talk went on after it.
    Dashed,
    /// Your turn, without a decision: answer the agent who spoke last.
    Neutral,
    /// Your word is the last: you wait on them (a discreet dotted line).
    Waiting,
    /// Not your turn, and not your word last.
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowState {
    pub band: Band,
    pub turn: Turn,
    pub glyph: Option<Glyph>,
    pub stripe: Stripe,
}

/// `user` is who reads the list: the one whose turn "you" means. The band,
/// the turn and the glyph are aiball's (`/api/inbox?v=tvty`); one it leaves
/// out, or does not name the way tvty knows, reads as nothing to show.
pub fn of(row: &TicketRow, user: &str) -> RowState {
    let band = row.band.and_then(Band::from_server).unwrap_or(Band::Open);
    let turn = match row.turn.as_deref() {
        Some("you") => Turn::You,
        Some("them") => Turn::Them,
        _ => Turn::Nobody,
    };
    let glyph = row.state_glyph.as_deref().and_then(Glyph::from_server);
    let mut stripe = stripe(turn, row.pending_decision(), row.pending_decision_is_latest);
    if stripe == Stripe::None && row.last_speaker.as_deref() == Some(user) {
        stripe = Stripe::Waiting;
    }
    RowState { band, turn, glyph, stripe }
}

fn stripe(turn: Turn, decision: bool, decision_is_latest: bool) -> Stripe {
    match turn {
        Turn::You if decision && decision_is_latest => Stripe::Solid,
        Turn::You if decision => Stripe::Dashed,
        Turn::You => Stripe::Neutral,
        _ => Stripe::None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Band, Glyph, Stripe, Turn, of};
    use crate::aiball::TicketRow;

    fn row() -> TicketRow {
        serde_json::from_value(serde_json::json!({
            "id": 1, "project": "demo", "title": "t", "status": "approved",
            "priority": "normal", "claimant": null, "assignee": null,
            "last_speaker": "david", "last_activity": null, "critical": null
        }))
        .unwrap()
    }

    #[test]
    fn a_fresh_decision_waits_on_you() {
        let mut r = row();
        r.band = Some(1);
        r.turn = Some("you".into());
        r.state_glyph = Some("plan".into());
        r.pending_plan = true;
        r.pending_decision_is_latest = true;
        let state = of(&r, "david");
        assert_eq!((state.band, state.turn, state.glyph, state.stripe), (Band::Decide, Turn::You, Some(Glyph::Plan), Stripe::Solid));
        // The talk went on after it.
        r.pending_decision_is_latest = false;
        assert_eq!(of(&r, "david").stripe, Stripe::Dashed);
    }

    #[test]
    fn your_turn_without_a_decision() {
        let mut r = row();
        r.band = Some(4);
        r.turn = Some("you".into());
        r.last_speaker = Some("demo-claude".into());
        assert_eq!(of(&r, "david").stripe, Stripe::Neutral);
    }

    #[test]
    fn your_word_last_you_wait() {
        let mut r = row();
        r.band = Some(4);
        r.turn = Some("them".into());
        assert_eq!(of(&r, "david").stripe, Stripe::Waiting);
        // Someone else's word last, not yours to move: no stripe.
        r.last_speaker = Some("demo-crew".into());
        assert_eq!(of(&r, "david").stripe, Stripe::None);
    }

    #[test]
    fn a_rejection_has_its_glyph() {
        let mut r = row();
        r.band = Some(3);
        r.turn = Some("them".into());
        r.state_glyph = Some("rejected".into());
        let state = of(&r, "david");
        assert_eq!((state.band, state.glyph), (Band::AgentOnIt, Some(Glyph::Rejected)));
    }

    #[test]
    fn what_aiball_leaves_out_shows_nothing() {
        let state = of(&row(), "demo-crew");
        assert_eq!((state.band, state.turn, state.glyph, state.stripe), (Band::Open, Turn::Nobody, None, Stripe::None));
    }
}
