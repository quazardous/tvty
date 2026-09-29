//! An agent's backlog in a project, as aiball ranks it for the agent —
//! read once for every view that shows it (the panel's sunk tickets, the
//! agent bar's list), and read again when it moves: one of the project's
//! tickets changed, the agent's bar changed (a wake recorded, a pause
//! begun), the first pause ended, back on the bus or aiball restarted.
//!
//! A view asks with [`request`] and reads with [`get`]; it observes the
//! store's entity ([`store`]) to hear when a read comes in.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui_kit::*;
use serde_json::{Value, json};

use crate::aiball::{Aiball, AgentBacklog};
use crate::bus::Signal;
use crate::kernel::signals::BusSignal;

/// Changes come in bursts (a ticket's upsert, its bar): read once after.
const SETTLE: Duration = Duration::from_millis(500);

/// Whose backlog, in which project.
pub type Key = (String, String);

/// What aiball answered: the backlog, or why not.
pub type Read = Result<AgentBacklog, String>;

/// The backlogs read, by agent and project.
pub struct BacklogStore {
    aiball: Aiball,
    entries: HashMap<Key, Read>,
    /// Reads under way, and those asked again while under way.
    reading: HashSet<Key>,
    again: HashSet<Key>,
    /// Bumped by each read of a key: the timer of an older one lets go.
    generation: HashMap<Key, u64>,
}

struct GlobalStore(Entity<BacklogStore>);

impl Global for GlobalStore {}

/// Sets the store up, listening to the board's and the link's signals.
pub fn init(aiball: Aiball, cx: &mut App) {
    let store = cx.new(|cx| {
        cx.subscribe(&crate::bus::bus(cx), |store: &mut BacklogStore, _, signal: &Signal, cx| match signal {
            Signal::TicketsChanged(project) => store.moved(|(_, p)| p == project, cx),
            Signal::BarChanged(agent) => store.moved(|(a, _)| a == agent, cx),
            Signal::Bus(BusSignal::Reconnected { .. } | BusSignal::EpochChanged) => store.moved(|_| true, cx),
            // The tickets read whole after a failure: whatever moved meanwhile.
            Signal::Bus(BusSignal::SubscriptionLive { kind, resynced: true }) if kind == "Tickets" => store.moved(|_| true, cx),
            _ => {}
        })
        .detach();
        BacklogStore { aiball, entries: HashMap::new(), reading: HashSet::new(), again: HashSet::new(), generation: HashMap::new() }
    });
    cx.set_global(GlobalStore(store));
}

/// The store: observe it to hear a read come in.
pub fn store(cx: &App) -> Entity<BacklogStore> {
    cx.global::<GlobalStore>().0.clone()
}

/// `agent`'s backlog in `project`, as read, if it was.
pub fn get(cx: &App, agent: &str, project: &str) -> Option<Read> {
    store(cx).read(cx).entries.get(&(agent.to_string(), project.to_string())).cloned()
}

/// `agent`'s backlog in `project` read, unless it is already (it is kept
/// up to date) or under way.
pub fn request(cx: &mut App, agent: &str, project: &str) {
    let key = (agent.to_string(), project.to_string());
    store(cx).update(cx, |store, cx| {
        if !store.entries.contains_key(&key) {
            store.read(key, Duration::ZERO, cx);
        }
    });
}

impl BacklogStore {
    /// The backlogs `which` picks may have moved: read again, once the
    /// burst has settled.
    fn moved(&mut self, which: impl Fn(&Key) -> bool, cx: &mut Context<Self>) {
        let keys: Vec<Key> = self.entries.keys().filter(|k| which(k)).cloned().collect();
        for key in keys {
            self.read(key, SETTLE, cx);
        }
    }

    /// Reads `key` after `wait`; one read at a time (asked meanwhile, it
    /// reads once more after).
    fn read(&mut self, key: Key, wait: Duration, cx: &mut Context<Self>) {
        if self.reading.contains(&key) {
            self.again.insert(key);
            return;
        }
        self.reading.insert(key.clone());
        let generation = self.generation.entry(key.clone()).or_default();
        *generation += 1;
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            if !wait.is_zero() {
                cx.background_executor().timer(wait).await;
            }
            let (agent, project) = key.clone();
            let read = cx
                .background_executor()
                .spawn(async move { aiball.agent_backlog(&agent, &project).map_err(|e| format!("{e:#}")) })
                .await;
            let _ = this.update(cx, |store, cx| {
                store.reading.remove(&key);
                let until = read.as_ref().ok().and_then(first_pause_end);
                store.entries.insert(key.clone(), read);
                cx.notify();
                if store.again.remove(&key) {
                    store.read(key, SETTLE, cx);
                } else if let Some(until) = until {
                    store.at_pause_end(key, until, cx);
                }
            });
        })
        .detach();
    }

    /// Reads `key` again as its first pause ends (`until`, seconds since
    /// the epoch): the ticket comes up again, and says so.
    fn at_pause_end(&mut self, key: Key, until: u64, cx: &mut Context<Self>) {
        let generation = self.generation.get(&key).copied().unwrap_or_default();
        let wait = Duration::from_secs(until.saturating_sub(crate::status::now()) + 1);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            let _ = this.update(cx, |store, cx| {
                // A read since: its own timer holds.
                if store.generation.get(&key).copied().unwrap_or_default() == generation {
                    store.read(key, Duration::ZERO, cx);
                }
            });
        })
        .detach();
    }

    /// As the debug control says it.
    pub fn said(&self) -> Value {
        self.entries
            .iter()
            .map(|((agent, project), read)| {
                let said = match read {
                    Ok(backlog) => json!({ "rows": backlog.rows.len(), "first_pause_end": first_pause_end(backlog) }),
                    Err(error) => json!({ "error": error }),
                };
                (format!("{agent}@{project}"), said)
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    }
}

/// When the first pause still running ends, in seconds since the epoch.
fn first_pause_end(backlog: &AgentBacklog) -> Option<u64> {
    let now = crate::status::now();
    backlog
        .rows
        .iter()
        .filter_map(|r| r.backlog_cooled_until.as_deref().and_then(crate::status::parse_time))
        .filter(|at| *at > now)
        .min()
}
