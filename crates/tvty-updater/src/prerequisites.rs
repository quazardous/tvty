//! What Terminal Velocity and aiball need on the machine before anything is
//! installed: one list for every system — the command looked for, what it is
//! for, how to install it. The updater checks it first (its window, and
//! `--install`), says what is missing and how to get it; on Linux nothing is
//! installed for the user (package managers differ, and ask for `sudo`).

use std::path::{Path, PathBuf};

/// Something that must be on the machine.
#[derive(Debug, PartialEq)]
pub struct Prerequisite {
    /// The command looked for on the `PATH`.
    pub command: &'static str,
    /// What needs it, said to the user.
    pub purpose: &'static str,
    /// Needed on Unix, on Windows.
    pub unix: bool,
    pub windows: bool,
    /// Its package by Linux package manager (dnf, apt, pacman, zypper), when
    /// a package manager has it.
    packages: Option<[&'static str; 4]>,
    /// How to install it on Unix when no package manager does.
    script: Option<&'static str>,
    /// Its winget id, on Windows.
    pub winget: Option<&'static str>,
}

/// The list, in the order they are said.
pub const ALL: &[Prerequisite] = &[
    Prerequisite {
        command: "git",
        purpose: "aiball (its install clones it)",
        unix: true,
        windows: true,
        packages: Some(["git", "git", "git", "git"]),
        script: None,
        winget: Some("Git.Git"),
    },
    Prerequisite {
        command: "node",
        purpose: "aiball (it runs on Node.js)",
        unix: true,
        windows: true,
        packages: Some(["nodejs npm", "nodejs npm", "nodejs npm", "nodejs npm"]),
        script: None,
        winget: Some("OpenJS.NodeJS.LTS"),
    },
    Prerequisite {
        command: "pwsh",
        purpose: "aiball's installer",
        unix: false,
        windows: true,
        packages: None,
        script: None,
        winget: Some("Microsoft.PowerShell"),
    },
    Prerequisite {
        command: "tmux",
        purpose: "the loops in tmux mode",
        unix: true,
        windows: false,
        packages: Some(["tmux", "tmux", "tmux", "tmux"]),
        script: None,
        winget: None,
    },
    Prerequisite {
        command: "psmux",
        purpose: "the loops in tmux mode",
        unix: false,
        windows: true,
        packages: None,
        script: None,
        winget: Some("marlocarlo.psmux"),
    },
    Prerequisite {
        command: "claude",
        purpose: "the agents themselves (Claude Code; run `claude` once to sign in)",
        unix: true,
        windows: true,
        packages: None,
        // Claude Code's own installer: no sudo.
        script: Some("curl -fsSL https://claude.ai/install.sh | bash"),
        winget: Some("Anthropic.ClaudeCode"),
    },
];

/// A Linux package manager.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Manager {
    Dnf,
    Apt,
    Pacman,
    Zypper,
}

impl Manager {
    /// The one of the distribution `os_release` (the text of
    /// `/etc/os-release`) describes, from its `ID` and `ID_LIKE`.
    pub fn of(os_release: &str) -> Option<Self> {
        let value = |key: &str| {
            os_release
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .map(|v| v.trim().trim_matches('"').to_lowercase())
                .unwrap_or_default()
        };
        let ids = format!("{} {}", value("ID"), value("ID_LIKE"));
        ids.split_whitespace().find_map(|id| match id {
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => Some(Self::Dnf),
            "debian" | "ubuntu" | "linuxmint" | "pop" => Some(Self::Apt),
            "arch" | "manjaro" | "endeavouros" => Some(Self::Pacman),
            "suse" | "opensuse" | "opensuse-tumbleweed" | "opensuse-leap" | "sles" => Some(Self::Zypper),
            _ => None,
        })
    }

    /// This machine's.
    pub fn here() -> Option<Self> {
        Self::of(&std::fs::read_to_string("/etc/os-release").ok()?)
    }

    fn install(self, packages: &str) -> String {
        match self {
            Self::Dnf => format!("sudo dnf install {packages}"),
            Self::Apt => format!("sudo apt install {packages}"),
            Self::Pacman => format!("sudo pacman -S {packages}"),
            Self::Zypper => format!("sudo zypper install {packages}"),
        }
    }
}

impl Prerequisite {
    /// Needed on this system.
    pub fn here(&self) -> bool {
        if cfg!(windows) { self.windows } else { self.unix }
    }

    /// The command that installs it, to run by hand: on Windows through
    /// winget; on Linux its own installer, or the distribution's package
    /// manager (`manager`; none known: said in words).
    pub fn how(&self, windows: bool, manager: Option<Manager>) -> String {
        if windows {
            return match self.winget {
                Some(id) => format!("winget install --id {id} --exact --source winget"),
                None => format!("install {}", self.command),
            };
        }
        if let Some(script) = self.script {
            return script.to_string();
        }
        let index = |m: Manager| match m {
            Manager::Dnf => 0,
            Manager::Apt => 1,
            Manager::Pacman => 2,
            Manager::Zypper => 3,
        };
        match (self.packages, manager) {
            (Some(packages), Some(manager)) => manager.install(packages[index(manager)]),
            _ => format!("install {} with your package manager", self.command),
        }
    }

    /// [`Prerequisite::how`] on this machine.
    pub fn how_here(&self) -> String {
        self.how(cfg!(windows), if cfg!(windows) { None } else { Manager::here() })
    }
}

/// `command` is found in one of `dirs` (as a program: `.exe`, `.cmd` too on
/// Windows).
fn found_in(command: &str, dirs: &[PathBuf], windows: bool) -> bool {
    let names: Vec<String> = if windows {
        ["", ".exe", ".cmd", ".bat"].iter().map(|ext| format!("{command}{ext}")).collect()
    } else {
        vec![command.to_string()]
    };
    dirs.iter().any(|dir| names.iter().any(|name| is_file(&dir.join(name))))
}

fn is_file(path: &Path) -> bool {
    path.is_file()
}

/// The folders programs are looked for in: the `PATH`, and `~/.local/bin`
/// (where Claude Code and tvty install themselves — a session started from
/// the desktop may not have it on its `PATH` yet).
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH").map(|path| std::env::split_paths(&path).collect()).unwrap_or_default();
    dirs.push(crate::bin_dir());
    dirs
}

/// What this system needs and does not have.
pub fn missing() -> Vec<&'static Prerequisite> {
    let dirs = search_dirs();
    ALL.iter().filter(|p| p.here() && !found_in(p.command, &dirs, cfg!(windows))).collect()
}

/// Says each missing prerequisite, what it is for and how to install it;
/// answers their commands.
pub fn report(say: &mut dyn FnMut(String)) -> Vec<&'static str> {
    let missing = missing();
    for p in &missing {
        say(format!("✗ {} is missing, for {}: {}", p.command, p.purpose, p.how_here()));
    }
    missing.iter().map(|p| p.command).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(command: &str) -> &'static Prerequisite {
        ALL.iter().find(|p| p.command == command).expect("in the list")
    }

    #[test]
    fn each_system_has_its_own_multiplexer_and_both_need_claude() {
        let unix: Vec<&str> = ALL.iter().filter(|p| p.unix).map(|p| p.command).collect();
        let windows: Vec<&str> = ALL.iter().filter(|p| p.windows).map(|p| p.command).collect();
        assert_eq!(unix, vec!["git", "node", "tmux", "claude"]);
        assert_eq!(windows, vec!["git", "node", "pwsh", "psmux", "claude"]);
        // Every Windows one installs through winget.
        assert!(ALL.iter().filter(|p| p.windows).all(|p| p.winget.is_some()));
    }

    #[test]
    fn a_distribution_is_told_by_its_id_or_what_it_is_like() {
        assert_eq!(Manager::of("NAME=\"Fedora Linux\"\nID=fedora\n"), Some(Manager::Dnf));
        assert_eq!(Manager::of("ID=ubuntu\nID_LIKE=debian\n"), Some(Manager::Apt));
        // Not known by its own id: by what it is like.
        assert_eq!(Manager::of("ID=nobara\nID_LIKE=\"rhel centos fedora\"\n"), Some(Manager::Dnf));
        assert_eq!(Manager::of("ID=cachyos\nID_LIKE=arch\n"), Some(Manager::Pacman));
        assert_eq!(Manager::of("ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n"), Some(Manager::Zypper));
        assert_eq!(Manager::of("ID=nixos\n"), None);
    }

    #[test]
    fn how_to_install_is_the_package_managers_command_or_its_own_installer() {
        assert_eq!(named("tmux").how(false, Some(Manager::Dnf)), "sudo dnf install tmux");
        assert_eq!(named("node").how(false, Some(Manager::Apt)), "sudo apt install nodejs npm");
        assert_eq!(named("git").how(false, Some(Manager::Pacman)), "sudo pacman -S git");
        // No package manager known: said in words.
        assert_eq!(named("tmux").how(false, None), "install tmux with your package manager");
        // Claude Code: its own installer, whatever the distribution.
        assert_eq!(named("claude").how(false, Some(Manager::Zypper)), "curl -fsSL https://claude.ai/install.sh | bash");
        // On Windows, winget.
        assert_eq!(named("claude").how(true, None), "winget install --id Anthropic.ClaudeCode --exact --source winget");
    }

    #[test]
    fn a_command_is_found_in_the_folders_searched() {
        let dir = std::env::temp_dir().join(format!("tvty-prerequisites-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("tmux"), "").unwrap();
        std::fs::write(dir.join("claude.cmd"), "").unwrap();
        let dirs = vec![dir.clone()];
        assert!(found_in("tmux", &dirs, false));
        assert!(!found_in("claude", &dirs, false));
        // On Windows a program has an extension.
        assert!(found_in("claude", &dirs, true));
        assert!(!found_in("git", &dirs, true));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
