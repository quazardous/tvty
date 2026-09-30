//! Which terminals there are: the tmux sessions on this machine, grouped by
//! project. A claude-loop session (`cl-*`) is matched to its aiball agent by
//! its directory, which gives it a project and a name; any other session
//! lands in a "tmux" group. The shells aiball's host holds go with the
//! project whose folder they started in, or in a "terminals" group. With
//! them, the open tickets of each project, as aiball pushes them (see
//! [`crate::live`]).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::aiball::{BarRead, Consumer, TicketRow};

#[derive(Clone, Debug, PartialEq)]
pub struct Terminal {
    /// The tmux session to attach to.
    pub session: String,
    /// The agent's name, or the session's when no agent owns it.
    pub label: String,
    /// The aiball agent running in it, if any.
    pub agent: Option<String>,
    /// Its Claude's state, as aiball has it.
    pub status: Option<Status>,
    /// The attach socket of a session a host holds: opened over it, not
    /// through tmux.
    pub attach: Option<String>,
}

/// A loop's Claude, as aiball centralises it.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub state: String,
    /// When it entered that state, in seconds since the epoch.
    pub since: Option<u64>,
    pub driver: String,
    pub online: bool,
    /// Its events not seen yet.
    pub unseen: u32,
    /// Where its Claude works.
    pub cwd: Option<String>,
    /// Its counters, as the daemon computes them.
    pub counters: Option<crate::aiball::AgentCounters>,
    /// The clients attached to its session, and those with the controls.
    pub clients: Option<u32>,
    pub interactive: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    /// Whether it is an aiball project (and has tickets).
    pub on_board: bool,
    pub terminals: Vec<Terminal>,
}

/// Everything the window shows besides the terminals themselves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Board {
    pub projects: Vec<Project>,
    /// Open tickets by project.
    pub tickets: HashMap<String, Vec<TicketRow>>,
    /// The critical ticket of each project, if any.
    pub critical: HashMap<String, u64>,
    /// The loop bar of each agent with a terminal, when its loop pushes one.
    pub bars: HashMap<String, BarRead>,
    /// The loops this machine knows, running or not.
    pub known: Vec<crate::loops::KnownLoop>,
    /// The agents aiball knows, with where they work: (agent, project, cwd).
    pub homes: Vec<(String, Option<String>, String)>,
    /// Behind a proxy node: the agents whose loop runs on aiball's hub.
    /// Read from here, never opened.
    pub hub: Vec<HubAgent>,
}

/// An agent whose loop runs on aiball's hub, seen from another machine.
#[derive(Clone, Debug, PartialEq)]
pub struct HubAgent {
    pub agent: String,
    pub project: Option<String>,
    pub status: Option<Status>,
}

/// aiball's hub, as it names machines (`node:<label>` for a proxy node).
pub const HUB: &str = "hub";

/// Where an agent works, seen from this machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Here,
    /// On aiball's hub, this machine being another.
    Hub,
    /// On another machine again (a node beside this one; seen from the
    /// hub, any node).
    Elsewhere,
}

/// Where `c` works. `mine` is this machine as aiball names it
/// (`bus.whoami`), and an agent's `machine` where its loop is connected:
/// the same, it is here. Without a loop connected an agent says no machine:
/// on the hub, aiball's `remote` stands (it is the hub's own view); on
/// another machine nothing says where it lives, and tvty does not guess —
/// it is not this machine's. An aiball that names no machine: its `remote`
/// stands.
pub fn place(c: &Consumer, mine: Option<&str>) -> Place {
    let as_aiball_says = if c.remote == Some(true) { Place::Elsewhere } else { Place::Here };
    let Some(mine) = mine else { return as_aiball_says };
    match c.machine.as_deref() {
        Some(machine) if machine == mine => Place::Here,
        Some(HUB) => Place::Hub,
        Some(_) => Place::Elsewhere,
        None if mine == HUB => as_aiball_says,
        None => Place::Elsewhere,
    }
}

/// The agents as this machine sees them, and those of the hub apart: an
/// agent working elsewhere keeps no session here — one another machine
/// holds cannot be attached from this one — and its folder is not offered.
fn placed(consumers: Vec<Consumer>, mine: Option<&str>) -> (Vec<Consumer>, Vec<HubAgent>) {
    let mut hub = Vec::new();
    let consumers = consumers
        .into_iter()
        .map(|mut c| {
            if c.kind != "agent" {
                return c;
            }
            let place = place(&c, mine);
            if place == Place::Hub {
                hub.push(HubAgent { agent: c.consumer_id.clone(), project: c.project.clone(), status: status_of(&c) });
            }
            c.remote = Some(place != Place::Here);
            if place != Place::Here {
                c.session = None;
            }
            c
        })
        .collect();
    hub.sort_by(|a, b| (a.project.as_deref().unwrap_or("").to_lowercase(), &a.agent).cmp(&(b.project.as_deref().unwrap_or("").to_lowercase(), &b.agent)));
    (consumers, hub)
}

const LOOP_PREFIX: &str = "cl-";
const OTHER_GROUP: &str = "tmux";
/// The group of the terminals the daemon's host holds (no agent).
pub const HOSTED_GROUP: &str = "terminals";
/// What names a hosted terminal's session, apart from tmux's.
pub const HOSTED_PREFIX: &str = "host:";

/// A project's terminals in the order they came: those already listed keep
/// their place, a new one goes last. `order` is the order kept (updated);
/// `prune` drops from it the sessions no longer there (not while the work
/// is still coming back at start: they are coming). Answers whether `order`
/// changed.
pub fn stack(terminals: &mut [Terminal], order: &mut Vec<String>, prune: bool) -> bool {
    let before = order.clone();
    if prune {
        order.retain(|s| terminals.iter().any(|t| t.session == *s));
    }
    for t in terminals.iter() {
        if !order.contains(&t.session) {
            order.push(t.session.clone());
        }
    }
    terminals.sort_by_key(|t| order.iter().position(|s| *s == t.session).unwrap_or(usize::MAX));
    *order != before
}

/// The words of a filter: what the user typed, lowercase, by word.
pub fn filter_words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_lowercase).collect()
}

/// Whether every word is found in one field or another of a row (its
/// project, its agent, its name, its folder); no word: every row.
pub fn found(words: &[String], fields: &[&str]) -> bool {
    let haystack = fields.iter().map(|f| f.to_lowercase()).collect::<Vec<_>>().join("\n");
    words.iter().all(|w| haystack.contains(w.as_str()))
}

/// The board, from what aiball pushed (`live`) and the tmux sessions of this
/// machine. Cheap: nothing is read from aiball.
/// `mine`: this machine, as aiball names it ([`place`]).
pub fn build(live: &crate::live::Live, sessions: Vec<(String, String)>, known: Vec<crate::loops::KnownLoop>, mine: Option<&str>) -> Board {
    let (consumers, hub) = placed(live.consumers(), mine);
    // Whom each loop's tmux session runs for, as aiball lists its loops.
    let owners: HashMap<String, String> = known.iter().filter(|l| !l.on_host()).filter_map(|l| Some((l.session(), l.agent.clone()?))).collect();
    let projects = group(sessions, &consumers, &owners);
    let mut board = Board { known, homes: homes(&consumers), hub, ..Default::default() };
    for project in projects.iter().filter(|p| p.on_board) {
        let tickets = live.tickets().get(&project.name).cloned().unwrap_or_default();
        if let Some(critical) = tickets.iter().find(|t| t.critical.is_some()) {
            board.critical.insert(project.name.clone(), critical.id);
        }
        board.tickets.insert(project.name.clone(), tickets);
    }
    for agent in projects.iter().flat_map(|p| p.terminals.iter().filter_map(|t| t.agent.as_ref())) {
        if let Some(bar) = live.bars().get(agent) {
            board.bars.insert(agent.clone(), bar.clone());
        }
    }
    let mut projects = projects;
    for terminal in projects.iter_mut().flat_map(|p| p.terminals.iter_mut()) {
        if let (Some(status), Some(bar)) = (terminal.status.as_mut(), terminal.agent.as_ref().and_then(|a| board.bars.get(a))) {
            loop_says(status, bar);
        }
    }
    board.projects = projects;
    // The daemon's own terminals: with the project they started in, after
    // its agents; the others together, before tmux's.
    let folders = folders(&board.homes, &board.known);
    let mut hosted = Vec::new();
    for t in live.terminals() {
        let terminal = Terminal {
            session: format!("{HOSTED_PREFIX}{}", t.name),
            label: t.name,
            agent: None,
            status: None,
            attach: Some(t.socket),
        };
        let Some(project) = t.cwd.as_deref().and_then(|cwd| project_at(cwd, &folders)) else {
            hosted.push(terminal);
            continue;
        };
        match board.projects.iter_mut().find(|p| p.name == project) {
            Some(p) => p.terminals.push(terminal),
            None => {
                // Its project has no agent here: listed in its place, by name.
                let at = board
                    .projects
                    .iter()
                    .position(|p| p.name == OTHER_GROUP || p.name.to_lowercase() > project.to_lowercase())
                    .unwrap_or(board.projects.len());
                board.projects.insert(at, Project { name: project.to_string(), on_board: true, terminals: vec![terminal] });
            }
        }
    }
    if !hosted.is_empty() {
        let at = board.projects.iter().position(|p| p.name == OTHER_GROUP).unwrap_or(board.projects.len());
        board.projects.insert(at, Project { name: HOSTED_GROUP.into(), on_board: false, terminals: hosted });
    }
    board
}

/// The folders of aiball's projects: where their agents work and their
/// loops run. (folder, project)
fn folders<'a>(homes: &'a [(String, Option<String>, String)], known: &'a [crate::loops::KnownLoop]) -> Vec<(&'a str, &'a str)> {
    let homes = homes.iter().filter_map(|(_, project, cwd)| Some((cwd.as_str(), project.as_deref()?)));
    let loops = known.iter().filter_map(|l| Some((l.cwd.as_str(), l.project.as_deref()?)));
    homes.chain(loops).collect()
}

/// The project a directory belongs to: the one whose folder holds it, the
/// deepest when folders nest.
fn project_at<'a>(cwd: &str, folders: &[(&str, &'a str)]) -> Option<&'a str> {
    let cwd = Path::new(cwd);
    folders
        .iter()
        .filter(|(folder, _)| cwd.starts_with(folder))
        .max_by_key(|(folder, _)| folder.len())
        .map(|(_, project)| *project)
}

/// The agents aiball knows with a working directory on this machine: an
/// agent working elsewhere has that machine's (a Windows path, say).
fn homes(consumers: &[Consumer]) -> Vec<(String, Option<String>, String)> {
    consumers
        .iter()
        .filter(|c| c.kind != "human" && c.remote != Some(true))
        .filter_map(|c| Some((c.consumer_id.clone(), c.project.clone(), c.cwd.clone()?)))
        .collect()
}

/// An agent's Claude, as aiball centralises it.
/// What a live loop's bar says wins over the agent's own state: the
/// loop knows its phase — a boot above all, while Claude, at its prompt,
/// already says idle (its state the one it left, the last time).
fn loop_says(status: &mut Status, bar: &crate::aiball::BarRead) {
    if bar.stale {
        return;
    }
    let phase_since = bar.phase_since;
    let bar = &bar.bar;
    if status.state != bar.phase {
        status.state = bar.phase.clone();
        status.since = None;
    }
    status.driver = bar.presence.clone();
    if let Some(started) = bar.boot.as_ref().and_then(|b| crate::status::parse_time(&b.started_at)) {
        status.since = Some(started);
    } else if let Some(changed) = phase_since {
        // The agent's own date may be older than the loop's phase (its
        // state the one left the last time): the later one.
        status.since = Some(status.since.map_or(changed, |since| since.max(changed)));
    }
}

fn status_of(c: &Consumer) -> Option<Status> {
    Some(Status {
        state: c.state.clone()?,
        since: c.state_since.as_deref().and_then(crate::status::parse_time),
        driver: c.state_human_word.clone().unwrap_or_default(),
        online: c.present.unwrap_or(false),
        unseen: c.ping_unseen.unwrap_or(0),
        cwd: c.cwd.clone(),
        counters: c.counters.clone(),
        clients: c.session.as_ref().and_then(|s| s.clients),
        interactive: c.session.as_ref().and_then(|s| s.interactive),
    })
}

/// Two ways a loop's terminal is there: aiball's host runs it (opened over
/// its attach socket), or claude-loop runs it in tmux (opened through tmux).
fn group(sessions: Vec<(String, String)>, consumers: &[Consumer], owners: &HashMap<String, String>) -> Vec<Project> {
    let mut projects: BTreeMap<String, (bool, Vec<Terminal>)> = BTreeMap::new();
    // The agents aiball's host runs.
    let hosted: Vec<(&Consumer, &str)> = consumers
        .iter()
        .filter(|c| c.kind == "agent")
        .filter_map(|c| Some((c, c.session.as_ref()?.socket()?)))
        .collect();
    for (c, socket) in &hosted {
        let (project, on_board) = match &c.project {
            Some(project) => (project.clone(), true),
            None => (c.cwd.as_deref().map(basename).unwrap_or_else(|| c.consumer_id.clone()), false),
        };
        let entry = projects.entry(project).or_default();
        entry.0 |= on_board;
        entry.1.push(Terminal {
            session: format!("{HOSTED_PREFIX}{}", c.consumer_id),
            label: c.consumer_id.clone(),
            attach: Some(socket.to_string()),
            agent: Some(c.consumer_id.clone()),
            status: status_of(c),
        });
    }
    for (session, path) in sessions {
        // A loop's agent: the one aiball says runs in it, else the one its
        // plate names, else the one working in its folder — a lead and its
        // crew share one, so the folder comes last.
        let agent = session
            .starts_with(LOOP_PREFIX)
            .then(|| {
                // An agent working on another machine owns no session here.
                let agents = || consumers.iter().filter(|c| c.kind == "agent" && c.remote != Some(true));
                agents()
                    .find(|c| c.session.as_ref().and_then(|s| s.tmux.as_deref()) == Some(session.as_str()))
                    .or_else(|| owners.get(&session).and_then(|owner| agents().find(|c| c.consumer_id == *owner)))
                    .or_else(|| agents().find(|c| c.cwd.as_deref() == Some(path.as_str())))
            })
            .flatten();
        // One place per agent: the host's, when both are seen for a moment.
        if agent.is_some_and(|a| hosted.iter().any(|(h, _)| h.consumer_id == a.consumer_id)) {
            continue;
        }
        let (project, on_board, label) = match agent {
            Some(c) => match &c.project {
                Some(project) => (project.clone(), true, c.consumer_id.clone()),
                None => (basename(&path), false, c.consumer_id.clone()),
            },
            None if session.starts_with(LOOP_PREFIX) => (basename(&path), false, session.clone()),
            None => (OTHER_GROUP.to_string(), false, session.clone()),
        };
        let entry = projects.entry(project).or_default();
        entry.0 |= on_board;
        entry.1.push(Terminal {
            session,
            label,
            attach: None,
            agent: agent.map(|c| c.consumer_id.clone()),
            status: agent.and_then(status_of),
        });
    }
    // Plain tmux sessions last: the projects are what tvty is for.
    let other = projects.remove(OTHER_GROUP);
    let mut list: Vec<Project> = projects
        .into_iter()
        .map(|(name, (on_board, terminals))| Project {
            name,
            on_board,
            terminals,
        })
        .collect();
    list.sort_by_key(|p| p.name.to_lowercase());
    if let Some((_, terminals)) = other {
        list.push(Project {
            name: OTHER_GROUP.into(),
            on_board: false,
            terminals,
        });
    }
    list
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// `(session name, start directory)` of every tmux session.
pub fn tmux_sessions() -> Vec<(String, String)> {
    let Ok(output) = crate::mux::command(&["ls", "-F", "#{session_name}\t#{session_path}"]).output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(name, path)| (name.to_string(), path.to_string()))
        .collect()
}

/// A pane's screen as `tmux capture-pane -e` prints it, colours included.
/// Blocking. A session's window size, in cells: a card watching it live
/// needs to be as large to see all of it.
pub fn window_size(session: &str) -> Option<(u16, u16)> {
    let output = crate::mux::command(&["display", "-p", "-t", &format!("={session}:"), "#{window_width} #{window_height}"])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (columns, lines) = text.trim().split_once(' ')?;
    Some((columns.parse().ok()?, lines.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::{HOSTED_PREFIX, HUB, Place, Status, filter_words, found, group, homes, loop_says, place, placed, project_at};
    use std::collections::HashMap;

    #[test]
    fn a_live_loops_bar_says_its_phase() {
        let mut status = Status {
            state: "idle".into(),
            since: Some(1),
            driver: "loop".into(),
            online: true,
            unseen: 0,
            cwd: None,
            counters: None,
            clients: None,
            interactive: None,
        };
        let bar: crate::aiball::BarRead = serde_json::from_value(serde_json::json!({
            "stale": false,
            "bar": { "phase": "boot", "presence": "boot", "afk": { "mode": "off", "expires_at": null },
                     "prompt": { "visible": true, "has_input": false }, "human_typing": false,
                     "marker": { "info": null, "health_prompt": false, "resume_picker": false, "resume_mode_picker": false },
                     "alerts": { "link_down": false, "daemon_down": false, "not_logged_in": false, "trust_dialog": false,
                                 "api_unreachable": false, "restart_needed": false },
                     "proxy_alive": true, "zen": false, "counters": null, "next_wake_at": null,
                     "boot": { "started_at": "2026-09-27T18:42:23.294Z", "deadline_at": "2026-09-27T18:42:53.294Z" },
                     "host": "external" }
        }))
        .unwrap();
        loop_says(&mut status, &bar);
        assert_eq!((status.state.as_str(), status.driver.as_str()), ("boot", "boot"));
        assert_eq!(status.since, crate::status::parse_time("2026-09-27T18:42:23.294Z"));
        // A stale bar says nothing: the loop is gone.
        let mut idle = Status { state: "idle".into(), since: Some(1), ..status.clone() };
        loop_says(&mut idle, &crate::aiball::BarRead { stale: true, ..bar.clone() });
        assert_eq!((idle.state.as_str(), idle.since), ("idle", Some(1)));
        // Out of its boot, idle since the loop said so, not since an older
        // state of the agent's.
        let mut after = Status { state: "idle".into(), since: Some(1), ..status.clone() };
        let mut out = bar;
        out.bar.phase = "idle".into();
        out.bar.boot = None;
        out.phase_since = Some(500);
        loop_says(&mut after, &out);
        assert_eq!(after.since, Some(500));
    }

    #[test]
    fn a_filter_finds_rows_by_every_word() {
        let words = filter_words("  Demo CREW ");
        assert_eq!(words, ["demo", "crew"]);
        assert!(found(&words, &["demo", "demo-crew", "/tmp/crew"]));
        assert!(!found(&words, &["demo", "demo-claude"]));
        // Words may sit in different fields.
        assert!(found(&filter_words("tvty lead"), &["tvty", "lead-agent"]));
        assert!(found(&[], &["anything"]));
    }
    use crate::aiball::Consumer;
    use serde_json::json;

    fn agent(id: &str, cwd: &str, session: serde_json::Value) -> Consumer {
        serde_json::from_value(json!({ "consumer_id": id, "kind": "agent", "cwd": cwd, "project": "demo",
                                        "state": "idle", "session": session })).unwrap()
    }

    #[test]
    fn an_agent_is_here_on_the_hub_or_elsewhere_by_the_machine_aiball_names() {
        let on = |machine: Option<&str>, remote: Option<bool>| {
            let mut c = agent("a", "/w/a", json!(null));
            c.machine = machine.map(str::to_string);
            c.remote = remote;
            c
        };
        let node = Some("node:laptop");
        // Behind a node: its own, the hub's, a node beside it.
        assert_eq!(place(&on(Some("node:laptop"), Some(true)), node), Place::Here);
        assert_eq!(place(&on(Some(HUB), Some(false)), node), Place::Hub);
        assert_eq!(place(&on(Some("node:other"), Some(true)), node), Place::Elsewhere);
        // No loop connected: nothing says where it lives, not guessed.
        assert_eq!(place(&on(None, Some(true)), node), Place::Elsewhere);
        assert_eq!(place(&on(None, Some(false)), node), Place::Elsewhere);
        // On the hub: a node's agent is elsewhere; without a loop, aiball's
        // `remote` stands (it is the hub's view).
        assert_eq!(place(&on(Some(HUB), None), Some(HUB)), Place::Here);
        assert_eq!(place(&on(Some("node:laptop"), Some(true)), Some(HUB)), Place::Elsewhere);
        assert_eq!(place(&on(None, Some(true)), Some(HUB)), Place::Elsewhere);
        assert_eq!(place(&on(None, None), Some(HUB)), Place::Here);
        // An aiball that names no machine: its `remote` stands.
        assert_eq!(place(&on(None, Some(true)), None), Place::Elsewhere);
        assert_eq!(place(&on(None, None), None), Place::Here);
    }

    #[test]
    fn behind_a_node_the_hubs_agents_are_apart_and_keep_no_session_here() {
        let mut hub = agent("hub-agent", "/srv/app", json!({ "running": true, "attach": { "socket": "/h/hub/attach.sock" } }));
        hub.machine = Some(HUB.into());
        let mut mine = agent("mine", "/w/app", json!(null));
        mine.machine = Some("node:laptop".into());
        mine.remote = Some(true);
        let mut beside = agent("beside", "C:\\w\\app", json!(null));
        beside.machine = Some("node:other".into());
        let (seen, on_hub) = placed(vec![hub, mine, beside], Some("node:laptop"));
        assert_eq!(on_hub.iter().map(|h| h.agent.as_str()).collect::<Vec<_>>(), ["hub-agent"], "the hub's alone: not a node beside");
        assert!(seen[0].session.is_none(), "a session the hub holds cannot be attached from here");
        assert_eq!(seen.iter().map(|c| c.remote).collect::<Vec<_>>(), [Some(true), Some(false), Some(true)]);
        // Shown here: this machine's session alone, and its folder.
        let projects = group(vec![("cl-mine".to_string(), "/w/app".to_string())], &seen, &HashMap::new());
        let labels: Vec<&str> = projects.iter().flat_map(|p| &p.terminals).map(|t| t.label.as_str()).collect();
        assert_eq!(labels, ["mine"]);
        assert_eq!(homes(&seen).iter().map(|h| h.0.as_str()).collect::<Vec<_>>(), ["mine"]);
    }

    #[test]
    fn an_agent_on_the_host_opens_over_its_socket_and_one_in_claude_loop_through_tmux() {
        let consumers = vec![
            agent("hosted", "/w/hosted", json!({ "running": true, "attach": { "socket": "/h/hosted/attach.sock" } })),
            agent("looped", "/w/looped", json!(null)),
        ];
        let tmux = vec![
            ("cl-looped".to_string(), "/w/looped".to_string()),
            // The host's agent seen in tmux a moment too: the host wins.
            ("cl-hosted".to_string(), "/w/hosted".to_string()),
        ];
        let projects = group(tmux, &consumers, &HashMap::new());
        let demo = projects.iter().find(|p| p.name == "demo").unwrap();
        let by = |label: &str| demo.terminals.iter().filter(|t| t.label == label).collect::<Vec<_>>();
        let hosted = by("hosted");
        assert_eq!(hosted.len(), 1);
        assert_eq!(hosted[0].session, format!("{HOSTED_PREFIX}hosted"));
        assert_eq!(hosted[0].attach.as_deref(), Some("/h/hosted/attach.sock"));
        let looped = by("looped");
        assert_eq!((looped[0].session.as_str(), looped[0].attach.is_none()), ("cl-looped", true));
    }

    #[test]
    fn a_loop_in_a_shared_folder_is_its_own_agents() {
        // A lead and its crew in one folder, each loop in tmux.
        let consumers = vec![agent("lead", "/w/app", json!(null)), agent("crew", "/w/app", json!({ "running": true, "tmux": "cl-b" }))];
        let tmux = vec![("cl-a".to_string(), "/w/app".to_string()), ("cl-b".to_string(), "/w/app".to_string()), ("cl-c".to_string(), "/w/app".to_string())];
        let owners = HashMap::from([("cl-c".to_string(), "lead".to_string())]);
        let projects = group(tmux, &consumers, &owners);
        let agent_of = |session: &str| {
            projects.iter().flat_map(|p| &p.terminals).find(|t| t.session == session).and_then(|t| t.agent.clone())
        };
        // aiball says: crew in cl-b; the plate says: lead in cl-c; cl-a, by its folder only.
        assert_eq!(agent_of("cl-b").as_deref(), Some("crew"));
        assert_eq!(agent_of("cl-c").as_deref(), Some("lead"));
        assert_eq!(agent_of("cl-a").as_deref(), Some("lead"));
    }

    #[test]
    fn an_agents_counters_come_from_the_daemon() {
        let mut counted: Consumer = serde_json::from_value(json!({ "consumer_id": "crew", "kind": "agent", "cwd": "/w/crew",
            "project": "demo", "state": "idle", "session": null,
            "counters": { "open": 7, "actionable": 2, "backlog": 3, "events": 1, "computed_at": "2026-09-27T13:00:00Z" } }))
        .unwrap();
        let projects = group(vec![("cl-crew".to_string(), "/w/crew".to_string())], std::slice::from_ref(&counted), &HashMap::new());
        let status = projects[0].terminals[0].status.clone().unwrap();
        assert_eq!(status.counters, Some(crate::aiball::AgentCounters { open: Some(7), backlog: Some(3), events: Some(1) }));
        // Not computed yet: none.
        counted.counters = None;
        let projects = group(vec![("cl-crew".to_string(), "/w/crew".to_string())], &[counted], &HashMap::new());
        assert_eq!(projects[0].terminals[0].status.as_ref().unwrap().counters, None);
    }

    #[test]
    fn an_agent_on_another_machine_gives_no_folder_here() {
        let mut far: Consumer = agent("win", "C:\\Users\\x\\app", json!(null));
        far.remote = Some(true);
        let near = agent("here", "/w/app", json!(null));
        let homes = homes(&[far, near]);
        assert_eq!(homes, vec![("here".to_string(), Some("demo".to_string()), "/w/app".to_string())]);
    }

    #[test]
    fn a_terminal_goes_with_the_project_it_started_in() {
        let folders = [("/w/app", "app"), ("/w/app/sub", "sub"), ("/w/other", "other")];
        assert_eq!(project_at("/w/app", &folders), Some("app"));
        assert_eq!(project_at("/w/app/src/deep", &folders), Some("app"));
        // Nested folders: the deepest decides.
        assert_eq!(project_at("/w/app/sub/x", &folders), Some("sub"));
        // A name that only begins like a folder is not in it.
        assert_eq!(project_at("/w/apple", &folders), None);
        assert_eq!(project_at("/home", &folders), None);
    }

    #[test]
    fn a_new_session_goes_last_and_the_others_keep_their_place() {
        let t = |s: &str| super::Terminal { session: s.into(), label: s.into(), attach: None, agent: None, status: None };
        let mut order = vec!["b".to_string(), "a".to_string()];
        // aiball lists them its way (the new "c" first): the order kept wins.
        let mut terminals = vec![t("c"), t("a"), t("b")];
        assert!(super::stack(&mut terminals, &mut order, true));
        assert_eq!(terminals.iter().map(|t| t.session.as_str()).collect::<Vec<_>>(), ["b", "a", "c"]);
        // One gone frees its place; back, it goes last.
        let mut terminals = vec![t("c"), t("b")];
        super::stack(&mut terminals, &mut order, true);
        assert_eq!(order, ["b", "c"]);
        let mut terminals = vec![t("a"), t("c"), t("b")];
        super::stack(&mut terminals, &mut order, true);
        assert_eq!(terminals.iter().map(|t| t.session.as_str()).collect::<Vec<_>>(), ["b", "c", "a"]);
        // At start, one not back yet keeps its place.
        let mut order = vec!["x".to_string(), "y".to_string()];
        let mut terminals = vec![t("y")];
        assert!(!super::stack(&mut terminals, &mut order, false));
        assert_eq!(order, ["x", "y"]);
    }
}