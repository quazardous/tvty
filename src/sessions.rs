//! Which terminals there are: the tmux sessions on this machine, grouped by
//! project. A claude-loop session (`cl-*`) is matched to its aiball agent by
//! its directory, which gives it a project and a name; any other session
//! lands in a "tmux" group. With them, the open tickets of each project.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

use crate::aiball::{Aiball, Consumer, TicketRow};

#[derive(Clone, Debug, PartialEq)]
pub struct Terminal {
    /// The tmux session to attach to.
    pub session: String,
    /// The agent's name, or the session's when no agent owns it.
    pub label: String,
    /// The aiball agent running in it, if any.
    pub agent: Option<String>,
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
        ..Default::default()
    };
    for project in board.projects.iter().filter(|p| p.on_board) {
        match aiball.open_tickets(&project.name) {
            Ok(tickets) => {
                board.tickets.insert(project.name.clone(), tickets);
            }
            Err(error) => log::warn!("aiball: {error:#}"),
        }
        if let Ok(Some(id)) = aiball.critical(&project.name) {
            board.critical.insert(project.name.clone(), id);
        }
    }
    board
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
pub struct Capture {
    pub columns: usize,
    pub lines: usize,
    pub text: Vec<u8>,
}

/// Blocking. Reads a session's current pane without attaching to it — so
/// without resizing it, unlike opening its terminal.
pub fn capture(session: &str) -> Option<Capture> {
    let target = format!("={session}:");
    let tmux = |args: &[&str]| {
        Command::new("tmux")
            .args(args)
            .env_remove("TMUX")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| o.stdout)
    };
    let size = tmux(&["display", "-p", "-t", &target, "#{pane_width} #{pane_height}"])?;
    let size = String::from_utf8_lossy(&size);
    let (columns, lines) = size.trim().split_once(' ')?;
    let text = tmux(&["capture-pane", "-p", "-e", "-t", &target])?;
    Some(Capture {
        columns: columns.parse().ok()?,
        lines: lines.parse().ok()?,
        text,
    })
}
