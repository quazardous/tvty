//! Which terminals there are: the tmux sessions on this machine, grouped by
//! project. A claude-loop session (`cl-*`) is matched to its aiball agent by
//! its directory, which gives it a project and a name; any other session
//! lands in a "tmux" group. With them, the open tickets of each project, as
//! aiball pushes them (see [`crate::live`]).

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
    /// Its wait credit in its project, in minutes.
    pub credit: Option<i64>,
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
    board
}

/// The agents aiball knows with a working directory.
fn homes(consumers: &[Consumer]) -> Vec<(String, Option<String>, String)> {
    consumers
        .iter()
        .filter(|c| c.kind != "human")
        .filter_map(|c| Some((c.consumer_id.clone(), c.project.clone(), c.cwd.clone()?)))
        .collect()
}

fn group(sessions: Vec<(String, String)>, consumers: &[Consumer]) -> Vec<Project> {
    let mut projects: BTreeMap<String, (bool, Vec<Terminal>)> = BTreeMap::new();
    for (session, path) in sessions {
        let agent = session
            .starts_with(LOOP_PREFIX)
            .then(|| {
                consumers
                    .iter()
                    .find(|c| c.kind == "agent" && c.cwd.as_deref() == Some(path.as_str()))
            })
            .flatten();
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
            agent: agent.map(|c| c.consumer_id.clone()),
            status: agent.and_then(|c| {
                Some(Status {
                    state: c.state.clone()?,
                    since: c.state_since.as_deref().and_then(crate::status::parse_time),
                    driver: c.state_human_word.clone().unwrap_or_default(),
                    online: c.present.unwrap_or(false),
                    unseen: c.ping_unseen.unwrap_or(0),
                    credit: c
                        .wait_credit
                        .iter()
                        .flatten()
                        .find(|w| Some(&w.project) == c.project.as_ref())
                        .map(|w| w.balance),
                    cwd: c.cwd.clone(),
                })
            }),
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
