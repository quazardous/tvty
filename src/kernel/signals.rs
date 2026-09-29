//! Where tvty stands with aiball's bus, said as signals the views observe:
//! connected, dropped, connected again (and after how long), a subscription
//! failed or live again, aiball restarted (a new epoch: whatever is kept is
//! to be read again). The shell feeds this with what the wire and the
//! subscriptions say, and publishes each signal on the internal bus
//! (`crate::bus::Signal::Bus`); a view subscribes and reacts, none guesses.

use std::collections::{BTreeSet, VecDeque};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// A change in tvty's link to aiball.
#[derive(Clone, Debug, PartialEq)]
pub enum BusSignal {
    /// The first greeting of the daemon.
    Connected { version: u64, user: String },
    /// A connection that ran is gone.
    Disconnected { why: String },
    /// Greeted again, after a drop that lasted `after`.
    Reconnected { after: Duration },
    /// A subscription refused or unanswered: its lists may be behind.
    SubscriptionFailed { kind: String, error: String },
    /// A subscription answered; `resynced` when it had failed before (its
    /// lists came whole).
    SubscriptionLive { kind: String, resynced: bool },
    /// aiball restarted: another epoch. What is kept from it is stale.
    EpochChanged,
}

/// How many signals are kept for the debug control.
const RECENT: usize = 20;

/// The link as it stands, and the signals that led there.
#[derive(Debug, Default)]
pub struct BusState {
    connected: bool,
    /// Greeted at least once.
    ever: bool,
    down_since: Option<Instant>,
    epoch: Option<Value>,
    failing: BTreeSet<String>,
    recent: VecDeque<String>,
}

impl BusState {
    /// The daemon greeted this connection.
    pub fn hello(&mut self, version: u64, user: &str, now: Instant) -> Vec<BusSignal> {
        let signal = match (self.ever, self.down_since.take()) {
            (true, Some(down)) => BusSignal::Reconnected { after: now.saturating_duration_since(down) },
            // Greeted again with no drop between (another user): as a first.
            _ => BusSignal::Connected { version, user: user.to_string() },
        };
        self.connected = true;
        self.ever = true;
        self.keep(vec![signal])
    }

    /// A connection that ran is gone.
    pub fn dropped(&mut self, why: &str, now: Instant) -> Vec<BusSignal> {
        if !self.connected {
            return Vec::new();
        }
        self.connected = false;
        self.down_since = Some(now);
        self.keep(vec![BusSignal::Disconnected { why: why.to_string() }])
    }

    /// A subscription failed.
    pub fn failed(&mut self, kind: &str, error: &str) -> Vec<BusSignal> {
        self.failing.insert(kind.to_string());
        self.keep(vec![BusSignal::SubscriptionFailed { kind: kind.to_string(), error: error.to_string() }])
    }

    /// A subscription answered, in `epoch`.
    pub fn live(&mut self, kind: &str, epoch: Option<&Value>) -> Vec<BusSignal> {
        let resynced = self.failing.remove(kind);
        let mut signals = Vec::new();
        if let Some(epoch) = epoch {
            if self.epoch.as_ref().is_some_and(|known| known != epoch) {
                signals.push(BusSignal::EpochChanged);
            }
            self.epoch = Some(epoch.clone());
        }
        signals.push(BusSignal::SubscriptionLive { kind: kind.to_string(), resynced });
        self.keep(signals)
    }

    /// Greeted once, and down now.
    pub fn down(&self) -> bool {
        self.ever && !self.connected
    }

    /// The subscriptions failing now.
    pub fn failing(&self) -> Vec<String> {
        self.failing.iter().cloned().collect()
    }

    /// As the debug control says it.
    pub fn said(&self) -> Value {
        json!({ "connected": self.connected, "failing": self.failing(), "recent": self.recent })
    }

    fn keep(&mut self, signals: Vec<BusSignal>) -> Vec<BusSignal> {
        for signal in &signals {
            if self.recent.len() == RECENT {
                self.recent.pop_front();
            }
            self.recent.push_back(format!("{signal:?}"));
        }
        signals
    }
}

#[cfg(test)]
mod tests {
    use super::{BusSignal, BusState};
    use serde_json::json;
    use std::time::{Duration, Instant};

    #[test]
    fn a_drop_then_a_greeting_is_a_reconnection_after_how_long() {
        let mut s = BusState::default();
        let t0 = Instant::now();
        assert_eq!(s.hello(7, "david", t0), vec![BusSignal::Connected { version: 7, user: "david".into() }]);
        assert_eq!(s.dropped("reset", t0 + Duration::from_secs(1)), vec![BusSignal::Disconnected { why: "reset".into() }]);
        assert!(s.down());
        // Tries that fail while down say nothing more.
        assert!(s.dropped("again", t0 + Duration::from_secs(2)).is_empty());
        assert_eq!(s.hello(7, "david", t0 + Duration::from_secs(6)), vec![BusSignal::Reconnected { after: Duration::from_secs(5) }]);
        assert!(!s.down());
    }

    #[test]
    fn a_failed_subscription_comes_back_resynced_and_a_new_epoch_is_said() {
        let mut s = BusState::default();
        s.live("Tickets", Some(&json!("e1")));
        s.failed("Tickets", "no answer");
        assert_eq!(s.failing(), vec!["Tickets".to_string()]);
        assert_eq!(s.live("Tickets", Some(&json!("e1"))), vec![BusSignal::SubscriptionLive { kind: "Tickets".into(), resynced: true }]);
        assert!(s.failing().is_empty());
        // aiball restarted.
        assert_eq!(
            s.live("Board", Some(&json!("e2"))),
            vec![BusSignal::EpochChanged, BusSignal::SubscriptionLive { kind: "Board".into(), resynced: false }]
        );
        // Said once: the other subscriptions of that epoch do not say it again.
        assert_eq!(s.live("State", Some(&json!("e2"))), vec![BusSignal::SubscriptionLive { kind: "State".into(), resynced: false }]);
    }
}
