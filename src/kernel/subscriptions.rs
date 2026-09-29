//! The bus's subscriptions, each on its own: where it resumes from (its own
//! cursor, never another's), whether it is live, and when a failed one is
//! tried again. One place decides "resume" or "read it all again", and says
//! the gaps.
//!
//! A subscription that failed has no cursor any more: when it is tried
//! again, its value comes whole — the events it missed are not replayed on
//! top of a cursor that other subscriptions moved past them.

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// The first wait before a failed subscription is tried again; each failure
/// doubles it, up to [`RETRY_AT_MOST`].
const RETRY_FIRST: Duration = Duration::from_secs(3);
const RETRY_AT_MOST: Duration = Duration::from_secs(60);

/// Where a subscription is.
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// Asked for, no answer yet.
    Asked,
    /// Answered: its events come.
    Live,
    /// Refused or unanswered: tried again at `retry_at`.
    Failed { error: String, retry_at: Instant },
}

#[derive(Clone, Debug)]
struct Entry {
    /// The daemon's epoch and the last seq this subscription saw.
    cursor: Option<(Value, u64)>,
    state: State,
    /// Failures in a row.
    failures: u32,
}

impl Default for Entry {
    fn default() -> Self {
        Self { cursor: None, state: State::Asked, failures: 0 }
    }
}

/// Every subscription, by kind.
#[derive(Clone, Debug)]
pub struct Registry<K: Ord> {
    entries: BTreeMap<K, Entry>,
}

impl<K: Ord> Default for Registry<K> {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl<K: Ord + Copy + Debug> Registry<K> {
    /// What `kind` resumes from: its own cursor, as `since` takes it; none
    /// (its value whole) for one never live, or failed since.
    pub fn since(&self, kind: K) -> Option<Value> {
        let (epoch, seq) = self.entries.get(&kind)?.cursor.as_ref()?;
        Some(json!({ "epoch": epoch, "seq": seq }))
    }

    /// `kind` asked for.
    pub fn asked(&mut self, kind: K) {
        self.entries.entry(kind).or_default().state = State::Asked;
    }

    /// `kind` answered: live from `seq` of `epoch`.
    pub fn live(&mut self, kind: K, epoch: Option<Value>, seq: Option<u64>) {
        let entry = self.entries.entry(kind).or_default();
        if entry.failures > 0 {
            log::info!("aiball bus: {kind:?} live again, after {} failure(s)", entry.failures);
        }
        entry.state = State::Live;
        entry.failures = 0;
        if let (Some(epoch), Some(seq)) = (epoch, seq) {
            advance(entry, Some(epoch), seq);
        }
    }

    /// `kind` refused or unanswered at `now`: its cursor goes (it comes
    /// whole next time), and it is tried again later.
    pub fn failed(&mut self, kind: K, error: String, now: Instant) {
        let entry = self.entries.entry(kind).or_default();
        entry.failures += 1;
        entry.cursor = None;
        let wait = RETRY_FIRST.saturating_mul(1 << (entry.failures - 1).min(5)).min(RETRY_AT_MOST);
        log::warn!("aiball bus: subscribing to {kind:?} failed ({error}); tried again in {} s, read whole", wait.as_secs());
        entry.state = State::Failed { error, retry_at: now + wait };
    }

    /// An event of `kind`: its cursor, and only its, moves on.
    pub fn advance(&mut self, kind: K, seq: u64) {
        if let Some(entry) = self.entries.get_mut(&kind) {
            advance(entry, None, seq);
        }
    }

    /// Every cursor dropped: all is read whole next time (another user).
    pub fn forget(&mut self) {
        for entry in self.entries.values_mut() {
            entry.cursor = None;
        }
    }

    /// The failed subscriptions due to be tried again at `now`.
    pub fn due(&self, now: Instant) -> Vec<K> {
        self.entries
            .iter()
            .filter(|(_, e)| matches!(&e.state, State::Failed { retry_at, .. } if *retry_at <= now))
            .map(|(k, _)| *k)
            .collect()
    }

    /// When the next failed subscription is to be tried again.
    pub fn next_retry(&self) -> Option<Instant> {
        self.entries
            .values()
            .filter_map(|e| match &e.state {
                State::Failed { retry_at, .. } => Some(*retry_at),
                _ => None,
            })
            .min()
    }

    #[cfg(test)]
    pub fn state(&self, kind: K) -> Option<&State> {
        self.entries.get(&kind).map(|e| &e.state)
    }

    /// Each subscription as said to the debug control.
    pub fn said(&self, now: Instant) -> Value {
        self.entries
            .iter()
            .map(|(kind, e)| {
                let state = match &e.state {
                    State::Asked => json!("asked"),
                    State::Live => json!("live"),
                    State::Failed { error, retry_at } => {
                        json!({ "failed": error, "retry_in_s": retry_at.saturating_duration_since(now).as_secs() })
                    }
                };
                (format!("{kind:?}"), json!({ "state": state, "cursor": e.cursor.as_ref().map(|(_, seq)| seq), "failures": e.failures }))
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    }
}

/// `entry`'s cursor at `seq`: a new epoch starts it again, the same one only
/// moves it on.
fn advance(entry: &mut Entry, epoch: Option<Value>, seq: u64) {
    entry.cursor = match (entry.cursor.take(), epoch) {
        (Some((known, last)), Some(epoch)) if known == epoch => Some((known, seq.max(last))),
        (_, Some(epoch)) => Some((epoch, seq)),
        (Some((known, last)), None) => Some((known, seq.max(last))),
        (None, None) => None,
    };
}

#[cfg(test)]
mod tests {
    use super::{Registry, State};
    use serde_json::json;
    use std::time::{Duration, Instant};

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum K {
        Tickets,
        Board,
    }

    #[test]
    fn a_failed_subscription_does_not_resume_on_the_others_cursor() {
        let mut r = Registry::default();
        let now = Instant::now();
        r.live(K::Tickets, Some(json!("e1")), Some(10));
        r.live(K::Board, Some(json!("e1")), Some(10));
        // A reconnection: tickets fail, the board goes on and moves on.
        r.failed(K::Tickets, "no answer".into(), now);
        r.advance(K::Board, 25);
        assert_eq!(r.since(K::Board), Some(json!({ "epoch": "e1", "seq": 25 })));
        // Tried again: whole, not from the board's 25 (the events it missed
        // before 25 would be lost).
        assert_eq!(r.since(K::Tickets), None);
        assert!(r.due(now).is_empty());
        assert_eq!(r.due(now + Duration::from_secs(3)), vec![K::Tickets]);
        r.live(K::Tickets, Some(json!("e1")), Some(26));
        assert_eq!(r.state(K::Tickets), Some(&State::Live));
        assert_eq!(r.next_retry(), None);
    }

    #[test]
    fn failures_in_a_row_wait_longer_up_to_a_minute() {
        let mut r = Registry::default();
        let now = Instant::now();
        let waits: Vec<u64> = (0..7)
            .map(|_| {
                r.failed(K::Tickets, "x".into(), now);
                r.next_retry().unwrap().duration_since(now).as_secs()
            })
            .collect();
        assert_eq!(waits, vec![3, 6, 12, 24, 48, 60, 60]);
    }

    #[test]
    fn a_new_epoch_starts_the_cursor_again() {
        let mut r = Registry::default();
        r.live(K::Board, Some(json!("e1")), Some(40));
        r.live(K::Board, Some(json!("e2")), Some(3));
        assert_eq!(r.since(K::Board), Some(json!({ "epoch": "e2", "seq": 3 })));
        // Events only move it on.
        r.advance(K::Board, 2);
        assert_eq!(r.since(K::Board), Some(json!({ "epoch": "e2", "seq": 3 })));
    }
}
