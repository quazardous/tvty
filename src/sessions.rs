//! Which terminals there are: the tmux sessions on this machine, grouped by
//! project. A claude-loop session (`cl-*`) is matched to its aiball agent by
//! its directory, which gives it a project and a name; any other session
//! lands in a "tmux" group. The shells aiball's host holds go with the
//! project whose folder they started in, or in a "terminals" group. With
//! them, the open tickets of each project, as aiball pushes them (see
//! [`crate::live`]).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

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
}

const LOOP_PREFIX: &str = "cl-";
const OTHER_GROUP: &str = "tmux";
/// The group of the terminals the daemon's host holds (no agent).
pub const HOSTED_GROUP: &str = "terminals";
/// What names a hosted terminal's session, apart from tmux's.
pub const HOSTED_PREFIX: &str = "host:";

/// The board, from what aiball pushed (`live`) and the tmux sessions of this
/// machine. Cheap: nothing is read from aiball.
pub fn build(live: &crate::live::Live, sessions: Vec<(String, String)>, known: Vec<crate::loops::KnownLoop>) -> Board {
    let consumers = live.consumers();
    let projects = group(sessions, &consumers);
    let mut board = Board { known, homes: homes(&consumers), ..Default::default() };
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
fn status_of(c: &Consumer) -> Option<Status> {
    Some(Status {
        state: c.state.clone()?,
        since: c.state_since.as_deref().and_then(crate::status::parse_time),
        driver: c.state_human_word.clone().unwrap_or_default(),
        online: c.present.unwrap_or(false),
        unseen: c.ping_unseen.unwrap_or(0),
        cwd: c.cwd.clone(),
    })
}

/// Two ways a loop's terminal is there: aiball's host runs it (opened over
/// its attach socket), or claude-loop runs it in tmux (opened through tmux).
fn group(sessions: Vec<(String, String)>, consumers: &[Consumer]) -> Vec<Project> {
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
        let agent = session
            .starts_with(LOOP_PREFIX)
            .then(|| {
                consumers
                    .iter()
                    .find(|c| c.kind == "agent" && c.cwd.as_deref() == Some(path.as_str()))
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
    let Ok(output) = Command::new("tmux")
        .args(["ls", "-F", "#{session_name}\t#{session_path}"])
        .env_remove("TMUX")
        .output()
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
    let output = Command::new("tmux")
        .args(["display", "-p", "-t", &format!("={session}:"), "#{window_width} #{window_height}"])
        .env_remove("TMUX")
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (columns, lines) = text.trim().split_once(' ')?;
    Some((columns.parse().ok()?, lines.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::{HOSTED_PREFIX, group, homes, project_at};
    use crate::aiball::Consumer;
    use serde_json::json;

    fn agent(id: &str, cwd: &str, session: serde_json::Value) -> Consumer {
        serde_json::from_value(json!({ "consumer_id": id, "kind": "agent", "cwd": cwd, "project": "demo",
                                        "state": "idle", "session": session })).unwrap()
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
        let projects = group(tmux, &consumers);
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
}
