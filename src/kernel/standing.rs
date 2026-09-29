//! What steers each project's agents, as aiball keeps it — its standing
//! instruction and its wake focus (the 📢) — kept here for every view that
//! shows it: the 📢 card, the panel's icon, the projects' list mark. Read
//! whole once the board is live, then followed from aiball's word of each
//! change; read whole again after a gap on the bus or a restart of aiball;
//! a focus that applies read again as it ends (it lapses with no word).
//!
//! A view reads with [`get`], or the projects steered now from the
//! [`Steered`] global; it observes the store's entity ([`store`]) to hear a
//! change.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui_kit::*;
use serde_json::{Value, json};

use crate::aiball::{Aiball, Standing};
use crate::bus::Signal;
use crate::kernel::signals::BusSignal;

/// The projects something steers now: the ticket panel lights its 📢.
#[derive(Clone, Default)]
pub struct Steered(pub HashSet<String>);

impl Global for Steered {}

/// Every project's standing.
pub struct StandingStore {
    aiball: Aiball,
    standings: HashMap<String, Standing>,
    /// Read whole at least once.
    loaded: bool,
    reading: bool,
}

struct GlobalStore(Entity<StandingStore>);

impl Global for GlobalStore {}

/// Sets the store up, listening to the board's and the link's signals.
pub fn init(aiball: Aiball, cx: &mut App) {
    cx.set_global(Steered::default());
    let store = cx.new(|cx| {
        cx.subscribe(&crate::bus::bus(cx), |store: &mut StandingStore, _, signal: &Signal, cx| match signal {
            // The board live: read whole, the first time.
            Signal::Bus(BusSignal::SubscriptionLive { kind, .. }) if kind == "Board" && !store.loaded => store.load(cx),
            // A gap, or aiball restarted: what changed meanwhile.
            Signal::Bus(BusSignal::Reconnected { .. } | BusSignal::EpochChanged) => store.load(cx),
            Signal::StandingChanged(standing) => store.apply(standing.clone(), cx),
            _ => {}
        })
        .detach();
        StandingStore { aiball, standings: HashMap::new(), loaded: false, reading: false }
    });
    cx.set_global(GlobalStore(store));
}

/// The store: observe it to hear a change.
pub fn store(cx: &App) -> Entity<StandingStore> {
    cx.global::<GlobalStore>().0.clone()
}

/// `project`'s standing, as known.
pub fn get(cx: &App, project: &str) -> Option<Standing> {
    store(cx).read(cx).standings.get(project).cloned()
}

/// A standing read or set by a view (the 📢 card): kept.
pub fn apply(cx: &mut App, standing: Standing) {
    store(cx).update(cx, |store, cx| store.apply(standing, cx));
}

impl StandingStore {
    /// Every project's standing in one call, and the end of the focuses
    /// that apply.
    fn load(&mut self, cx: &mut Context<Self>) {
        if self.reading {
            return;
        }
        self.reading = true;
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx
                .background_executor()
                .spawn(async move {
                    let all = aiball.standings()?;
                    // A focus that applies: read whole (its tickets, its end).
                    Ok::<_, anyhow::Error>(
                        all.into_iter()
                            .map(|s| if s.focus_active { aiball.standing(&s.project).unwrap_or(s) } else { s })
                            .map(|s| (s.project.clone(), s))
                            .collect::<HashMap<_, _>>(),
                    )
                })
                .await;
            let _ = this.update(cx, |store, cx| {
                store.reading = false;
                match read {
                    Ok(read) => {
                        store.loaded = true;
                        for standing in read.values() {
                            store.watch_focus_end(standing, cx);
                        }
                        store.set(read, cx);
                    }
                    Err(error) => log::warn!("standing: {error:#}"),
                }
            });
        })
        .detach();
    }

    /// One project's standing, as aiball said it changed.
    fn apply(&mut self, standing: Standing, cx: &mut Context<Self>) {
        self.watch_focus_end(&standing, cx);
        let mut all = self.standings.clone();
        all.insert(standing.project.clone(), standing);
        self.set(all, cx);
    }

    /// A focus that applies lapses at its end with no word from aiball: the
    /// project read again then.
    fn watch_focus_end(&mut self, standing: &Standing, cx: &mut Context<Self>) {
        let Some(end) = standing.focus_until.as_deref().filter(|_| standing.focus_active).and_then(crate::status::parse_time) else { return };
        let wait = end.saturating_sub(crate::status::now()) + 1;
        let project = standing.project.clone();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(wait)).await;
            let read = cx.background_executor().spawn(async move { aiball.standing(&project) }).await;
            if let Ok(standing) = read {
                let _ = this.update(cx, |store, cx| store.apply(standing, cx));
            }
        })
        .detach();
    }

    fn set(&mut self, standings: HashMap<String, Standing>, cx: &mut Context<Self>) {
        if standings == self.standings {
            return;
        }
        cx.set_global(Steered(steered_of(&standings)));
        self.standings = standings;
        cx.notify();
    }

    /// As the debug control says it.
    pub fn said(&self) -> Value {
        json!({
            "loaded": self.loaded,
            "steered": steered_of(&self.standings),
            "projects": self.standings.len(),
        })
    }
}

/// The projects something steers now: a standing instruction, or a focus
/// that applies.
fn steered_of(standings: &HashMap<String, Standing>) -> HashSet<String> {
    standings.iter().filter(|(_, s)| s.active()).map(|(p, _)| p.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::steered_of;
    use crate::aiball::Standing;
    use std::collections::HashMap;

    #[test]
    fn a_project_is_steered_by_an_instruction_or_a_focus_that_applies() {
        let standing = |project: &str, prompt: Option<&str>, focus_active: bool| Standing {
            project: project.into(),
            standing_prompt: prompt.map(str::to_string),
            focus_active,
            ..Default::default()
        };
        let all: HashMap<String, Standing> = [
            standing("a", Some("light debugging"), false),
            standing("b", None, true),
            standing("c", None, false),
        ]
        .into_iter()
        .map(|s| (s.project.clone(), s))
        .collect();
        let mut steered: Vec<_> = steered_of(&all).into_iter().collect();
        steered.sort();
        assert_eq!(steered, vec!["a".to_string(), "b".to_string()]);
    }
}
