//! What a ticket row tells at a glance: its band in the list, whose turn it
//! is, one state glyph, and the stripe on its left. Computed here from the
//! row aiball's `/api/inbox` builds — a prototype of the rules proposed for
//! aiball itself (see docs/UX.md, "The ticket list").

use crate::aiball::TicketRow;

/// Where the row sorts: from "it is yours" to "it runs by itself".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    pub fn symbol(self) -> &'static str {
        match self {
            Glyph::Escalation => "!",
            Glyph::Plan => "◆",
            Glyph::Resolution | Glyph::ClosedResolved => "✓",
            Glyph::Wontfix => "✕",
            Glyph::StalledStep => "‖",
            Glyph::Step => "▶",
            Glyph::Rejected => "↺",
            Glyph::Closed => "⊘",
        }
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
    /// Not your turn.
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowState {
    pub band: Band,
    pub turn: Turn,
    pub glyph: Option<Glyph>,
    pub stripe: Stripe,
}

/// `user` is who reads the list: the one whose turn "you" means.
pub fn of(row: &TicketRow, user: &str) -> RowState {
    let moderation = row.status == "pending" || row.pending_comment_count > 0;
    let decision = row.pending_decision();
    let agent_on_it = row.holder().is_some() || row.latest_is_step;

    let band = if row.closed {
        Band::Closed
    } else if moderation {
        Band::Moderate
    } else if decision {
        Band::Decide
    } else if row.unread {
        Band::Unread
    } else if agent_on_it {
        Band::AgentOnIt
    } else {
        Band::Open
    };

    let turn = if row.closed {
        Turn::Nobody
    } else if moderation || decision {
        Turn::You
    } else if row.latest_is_step {
        Turn::Them
    } else {
        // aiball's rule (src/db/last-actor-gate.ts): yours when someone else
        // acted last, or when you are the only one on it. From the row we
        // only see comments: a ticket you filed that nobody answered is
        // yours. (Closes, reopens and decisions count for aiball, not here —
        // the reason to have aiball compute the turn itself.)
        match row.last_speaker.as_deref() {
            Some(speaker) if speaker == user && row.comment_count == 0 => Turn::You,
            Some(speaker) if speaker == user => Turn::Them,
            Some(_) => Turn::You,
            None => Turn::Nobody,
        }
    };

    // The last decision is what blocks, so it wins over a step posted
    // after it; closed states win over everything.
    let glyph = if row.closed {
        Some(if row.resolved { Glyph::ClosedResolved } else { Glyph::Closed })
    } else if row.pending_escalation {
        Some(Glyph::Escalation)
    } else if row.pending_plan {
        Some(Glyph::Plan)
    } else if row.pending_resolution {
        Some(Glyph::Resolution)
    } else if row.pending_wontfix {
        Some(Glyph::Wontfix)
    } else if row.stalled_step {
        Some(Glyph::StalledStep)
    } else if row.latest_is_step {
        Some(Glyph::Step)
    } else if row.latest_plan_rejected || row.latest_resolution_rejected {
        Some(Glyph::Rejected)
    } else {
        None
    };

    let stripe = match turn {
        Turn::You if decision && row.pending_decision_is_latest => Stripe::Solid,
        Turn::You if decision => Stripe::Dashed,
        Turn::You => Stripe::Neutral,
        _ => Stripe::None,
    };

    RowState { band, turn, glyph, stripe }
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
    fn you_spoke_last_it_is_their_turn() {
        let mut r = row();
        r.comment_count = 2;
        let state = of(&r, "david");
        assert_eq!((state.band, state.turn, state.glyph, state.stripe), (Band::Open, Turn::Them, None, Stripe::None));
    }

    #[test]
    fn alone_on_it_it_is_yours() {
        // Filed by you, nobody answered: aiball's rule says yours.
        let state = of(&row(), "david");
        assert_eq!((state.turn, state.stripe), (Turn::You, Stripe::Neutral));
    }

    #[test]
    fn an_agent_spoke_last_it_is_your_turn() {
        let mut r = row();
        r.last_speaker = Some("demo-claude".into());
        let state = of(&r, "david");
        assert_eq!((state.turn, state.stripe), (Turn::You, Stripe::Neutral));
    }

    #[test]
    fn a_fresh_plan_waits_on_you() {
        let mut r = row();
        r.pending_plan = true;
        r.pending_decision_is_latest = true;
        let state = of(&r, "david");
        assert_eq!((state.band, state.turn, state.glyph, state.stripe), (Band::Decide, Turn::You, Some(Glyph::Plan), Stripe::Solid));
        r.pending_decision_is_latest = false;
        assert_eq!(of(&r, "david").stripe, Stripe::Dashed);
    }

    #[test]
    fn a_decision_wins_over_a_later_step() {
        let mut r = row();
        r.pending_resolution = true;
        r.latest_is_step = true;
        let state = of(&r, "david");
        assert_eq!((state.glyph, state.turn), (Some(Glyph::Resolution), Turn::You));
    }

    #[test]
    fn a_step_is_the_agents_turn() {
        let mut r = row();
        r.latest_is_step = true;
        r.claimant = Some("demo-claude".into());
        r.last_speaker = Some("demo-claude".into());
        let state = of(&r, "david");
        assert_eq!((state.band, state.turn, state.glyph), (Band::AgentOnIt, Turn::Them, Some(Glyph::Step)));
        r.stalled_step = true;
        assert_eq!(of(&r, "david").glyph, Some(Glyph::StalledStep));
    }

    #[test]
    fn a_rejected_resolution_is_the_agents_turn() {
        let mut r = row();
        // The proposal, then your rejection: you spoke last.
        r.comment_count = 2;
        r.latest_resolution_rejected = true;
        let state = of(&r, "david");
        assert_eq!((state.glyph, state.turn), (Some(Glyph::Rejected), Turn::Them));
    }

    #[test]
    fn moderation_comes_first_and_closed_last() {
        let mut r = row();
        r.status = "pending".into();
        r.unread = true;
        assert_eq!(of(&r, "david").band, Band::Moderate);
        let mut r = row();
        r.closed = true;
        r.resolved = true;
        let state = of(&r, "david");
        assert_eq!((state.band, state.turn, state.glyph), (Band::Closed, Turn::Nobody, Some(Glyph::ClosedResolved)));
    }
}
