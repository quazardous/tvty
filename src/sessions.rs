//! Which terminals there are: the tmux sessions on this machine, grouped by
//! project. A claude-loop session (`cl-*`) is matched to its aiball agent by
//! its directory, which gives it a project and a name; any other session
//! lands in a "tmux" group. With them, the open tickets of each project.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use crate::aiball::{Aiball, BarRead, Consumer, TicketRow};

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

/// Blocking: runs `tmux ls` and asks aiball. Call it off the UI thread.
/// Learns on the way who the user is (see [`Aiball::find_user`]).
pub fn discover(aiball: &mut Aiball) -> Board {
    let consumers = match aiball.consumers() {
        Ok(consumers) => consumers,
        Err(error) => {
            log::warn!("aiball: {error:#}");
            Vec::new()
        }
    };
    aiball.find_user(&consumers);
    let projects = group(tmux_sessions(), &consumers);
    let mut board = Board {
        projects,
        known: crate::loops::known(),
        homes: homes(&consumers),
        ..Default::default()
    };
    for name in on_board(&board) {
        read_project(aiball, &mut board, &name);
    }
    read_bars(aiball, &mut board);
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

/// The loop bars of the agents that have a terminal here.
fn read_bars(aiball: &Aiball, board: &mut Board) {
    board.bars.clear();
    let agents: Vec<String> = board
        .projects
        .iter()
        .flat_map(|p| p.terminals.iter().filter_map(|t| t.agent.clone()))
        .collect();
    for agent in agents {
        match aiball.agent_bar(&agent) {
            Ok(Some(bar)) => {
                board.bars.insert(agent, bar);
            }
            Ok(None) => {}
            Err(error) => log::debug!("bar of {agent}: {error:#}"),
        }
    }
}

/// What moved on the board since the last read, from aiball's live feed.
#[derive(Debug, Default)]
pub struct Changes {
    /// Read everything again.
    pub all: bool,
    /// The agents (their state, their directory) or the tmux sessions.
    pub consumers: bool,
    /// Projects whose tickets moved.
    pub projects: HashSet<String>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        !self.all && !self.consumers && self.projects.is_empty()
    }
}

/// Blocking. Reads again only what moved: aiball serves one request at a
/// time, and a project's open tickets can cost it a second.
pub fn update(aiball: &mut Aiball, previous: &Board, changes: &Changes) -> Board {
    if changes.all {
        return discover(aiball);
    }
    let mut board = previous.clone();
    if changes.consumers {
        match aiball.consumers() {
            Ok(consumers) => {
                aiball.find_user(&consumers);
                board.projects = group(tmux_sessions(), &consumers);
                board.known = crate::loops::known();
                board.homes = homes(&consumers);
            }
            Err(error) => log::warn!("aiball: {error:#}"),
        }
        read_bars(aiball, &mut board);
    }
    for name in on_board(&board) {
        // A project newly shown has never been read.
        if !changes.projects.contains(&name) && board.tickets.contains_key(&name) {
            continue;
        }
        read_project(aiball, &mut board, &name);
    }
    board
}

fn on_board(board: &Board) -> Vec<String> {
    board
        .projects
        .iter()
        .filter(|p| p.on_board)
        .map(|p| p.name.clone())
        .collect()
}

/// A project's open tickets; its critical ticket comes with them.
fn read_project(aiball: &Aiball, board: &mut Board, name: &str) {
    match aiball.open_tickets(name) {
        Ok(tickets) => {
            match tickets.iter().find(|t| t.critical.is_some()) {
                Some(critical) => board.critical.insert(name.to_string(), critical.id),
                None => board.critical.remove(name),
            };
            board.tickets.insert(name.to_string(), tickets);
        }
        Err(error) => log::warn!("aiball: {error:#}"),
    }
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
