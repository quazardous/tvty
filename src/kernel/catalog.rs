//! What a project offers to choose from — every project, every agent, its
//! own agents, its tags, its open milestones — read once for every view
//! that needs it (the panel, the full-screen ticket, a new ticket), kept
//! here, and read again when it may have moved: after a minute, once back
//! on the bus, all of it when aiball restarted.
//!
//! A view asks with [`request`] and reads with [`get`]; it observes the
//! store's entity ([`store`]) to hear when a read comes in.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui_kit::*;
use serde_json::{Value, json};

use crate::aiball::Aiball;
use crate::kernel::signals::BusSignal;

/// How long a read is taken as it is; older, a request reads it again
/// (tags and milestones made elsewhere say nothing on the bus).
const FRESH: Duration = Duration::from_secs(60);

/// What a project offers to choose from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    pub project: String,
    pub projects: Vec<String>,
    pub agents: Vec<String>,
    /// The agents working on this project, sorted.
    pub own: Vec<String>,
    pub tags: Vec<String>,
    pub milestones: Vec<(u64, String)>,
}

impl Catalog {
    /// Every agent, the project's own first: (name, whether it is).
    pub fn agents_by_project(&self) -> Vec<(String, bool)> {
        let mut all: Vec<(String, bool)> = self.own.iter().map(|a| (a.clone(), true)).collect();
        all.extend(self.agents.iter().filter(|a| !self.own.contains(a)).map(|a| (a.clone(), false)));
        all
    }

    /// Read of aiball, blocking: off the UI thread.
    fn read(aiball: &Aiball, project: &str) -> Self {
        let (projects, agents) = aiball.projects_and_agents().unwrap_or_default();
        let mut own: Vec<String> = aiball
            .consumers()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.kind != "human" && c.project.as_deref() == Some(project))
            .map(|c| c.consumer_id)
            .collect();
        own.sort();
        Self {
            project: project.to_string(),
            projects,
            agents,
            own,
            tags: aiball.tag_catalog(project).unwrap_or_default(),
            milestones: aiball.milestones(project).unwrap_or_default(),
        }
    }
}

/// The catalogs read, by project.
pub struct CatalogStore {
    aiball: Aiball,
    entries: HashMap<String, (Catalog, Instant)>,
    /// Reads under way.
    reading: HashSet<String>,
}

struct GlobalStore(Entity<CatalogStore>);

impl Global for GlobalStore {}

/// Sets the store up, listening to the link's signals.
pub fn init(aiball: Aiball, cx: &mut App) {
    let store = cx.new(|cx| {
        cx.subscribe(&crate::bus::bus(cx), |store: &mut CatalogStore, _, signal: &crate::bus::Signal, cx| match signal {
            // aiball restarted: nothing read of it holds.
            crate::bus::Signal::Bus(BusSignal::EpochChanged) => store.read_all_again(true, cx),
            // Back after a gap: what moved meanwhile is read again.
            crate::bus::Signal::Bus(BusSignal::Reconnected { .. }) => store.read_all_again(false, cx),
            _ => {}
        })
        .detach();
        CatalogStore { aiball, entries: HashMap::new(), reading: HashSet::new() }
    });
    cx.set_global(GlobalStore(store));
}

/// The store: observe it to hear a read come in.
pub fn store(cx: &App) -> Entity<CatalogStore> {
    cx.global::<GlobalStore>().0.clone()
}

/// `project`'s catalog as read, if it was.
pub fn get(cx: &App, project: &str) -> Option<Catalog> {
    store(cx).read(cx).entries.get(project).map(|(c, _)| c.clone())
}

/// `project`'s catalog read, unless read a moment ago or under way.
pub fn request(cx: &mut App, project: &str) {
    let project = project.to_string();
    store(cx).update(cx, |store, cx| store.request(project, cx));
}

impl CatalogStore {
    fn request(&mut self, project: String, cx: &mut Context<Self>) {
        let age = self.entries.get(&project).map(|(_, at)| at.elapsed());
        if !needs_read(age, self.reading.contains(&project)) {
            return;
        }
        self.reading.insert(project.clone());
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let asked = project.clone();
            let catalog = cx.background_executor().spawn(async move { Catalog::read(&aiball, &asked) }).await;
            let _ = this.update(cx, |store, cx| {
                store.reading.remove(&project);
                store.entries.insert(project, (catalog, Instant::now()));
                cx.notify();
            });
        })
        .detach();
    }

    /// Every catalog read again; `drop`: forgotten meanwhile (stale).
    fn read_all_again(&mut self, drop: bool, cx: &mut Context<Self>) {
        let projects: Vec<String> = self.entries.keys().cloned().collect();
        for project in projects {
            if drop {
                self.entries.remove(&project);
            } else if let Some((_, at)) = self.entries.get_mut(&project) {
                *at = Instant::now() - FRESH;
            }
            self.request(project, cx);
        }
        cx.notify();
    }

    /// As the debug control says it.
    pub fn said(&self) -> Value {
        self.entries
            .iter()
            .map(|(p, (c, at))| (p.clone(), json!({ "age_s": at.elapsed().as_secs(), "agents": c.agents.len(), "own": c.own, "tags": c.tags.len(), "milestones": c.milestones.len() })))
            .collect::<serde_json::Map<_, _>>()
            .into()
    }
}

/// Whether a catalog `age` old (never read: none) is read now.
fn needs_read(age: Option<Duration>, reading: bool) -> bool {
    !reading && age.is_none_or(|age| age >= FRESH)
}

#[cfg(test)]
mod tests {
    use super::{Catalog, needs_read};
    use std::time::Duration;

    #[test]
    fn a_catalog_is_read_once_then_again_when_a_minute_old() {
        assert!(needs_read(None, false));
        assert!(!needs_read(None, true), "one read at a time");
        assert!(!needs_read(Some(Duration::from_secs(5)), false));
        assert!(needs_read(Some(Duration::from_secs(60)), false));
    }

    #[test]
    fn the_projects_own_agents_come_first() {
        let c = Catalog { own: vec!["b".into()], agents: vec!["a".into(), "b".into(), "c".into()], ..Default::default() };
        assert_eq!(c.agents_by_project(), vec![("b".into(), true), ("a".into(), false), ("c".into(), false)]);
    }
}
