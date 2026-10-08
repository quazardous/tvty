//! What Terminal Velocity and aiball need on the machine before anything is
//! installed: one list for every system — the command looked for, what it is
//! for, how to install it. The updater checks it first (its window, and
//! `--install`), says what is missing and how to get it. On Linux it installs
//! by itself what needs no `sudo` (Claude Code, by its own installer); the
//! rest is the distribution's package manager's, named to the user. On
//! Windows it installs all of it, through winget.

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
    // On Windows a program may be an alias `is_file` cannot follow (pwsh's,
    // from the Store): its own entry, not a folder, is enough.
    path.is_file() || (cfg!(windows) && path.symlink_metadata().is_ok_and(|entry| !entry.is_dir()))
}

/// The folders programs are looked for in: the `PATH`, and `~/.local/bin`
/// (where Claude Code and tvty install themselves — a session started from
/// the desktop may not have it on its `PATH` yet). On Windows the `PATH`
/// the registry holds now comes first: an installer adds its folder there,
/// which a running program does not see — what winget installed a moment
/// ago is found at once.
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = registry_path();
    dirs.extend(std::env::var_os("PATH").map(|path| std::env::split_paths(&path).collect::<Vec<_>>()).unwrap_or_default());
    dirs.push(crate::bin_dir());
    dirs
}

/// [`search_dirs`] as a `PATH`: what the updater's own tools (git, pwsh,
/// aiball's command) are found by and run with.
pub fn search_path() -> std::ffi::OsString {
    std::env::join_paths(search_dirs().into_iter().filter(|dir| !dir.as_os_str().is_empty())).unwrap_or_default()
}

/// The machine's and the user's `PATH` as the registry has them (Windows).
#[cfg(windows)]
fn registry_path() -> Vec<PathBuf> {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    let machine = registry_text(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment", "Path");
    let user = registry_text(HKEY_CURRENT_USER, "Environment", "Path");
    [machine, user].into_iter().flatten().flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>()).collect()
}

#[cfg(not(windows))]
fn registry_path() -> Vec<PathBuf> {
    Vec::new()
}

/// A text value of the registry, its `%VARIABLES%` expanded.
#[cfg(windows)]
fn registry_text(root: windows_sys::Win32::System::Registry::HKEY, key: &str, name: &str) -> Option<String> {
    use windows_sys::Win32::System::Registry::{RRF_RT_REG_SZ, RegGetValueW};
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let (key, name) = (wide(key), wide(name));
    let mut bytes: u32 = 0;
    // SAFETY: plain Win32 calls on buffers that outlive them; the first asks
    // the size, the second fills a buffer larger than that size.
    // RRF_RT_REG_SZ also takes a REG_EXPAND_SZ value, expanded.
    unsafe {
        if RegGetValueW(root, key.as_ptr(), name.as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), std::ptr::null_mut(), &mut bytes) != 0 {
            return None;
        }
        // Expanding may need more room than the size first said.
        let mut text = vec![0u16; bytes as usize / 2 + 4096];
        let mut bytes = (text.len() * 2) as u32;
        if RegGetValueW(root, key.as_ptr(), name.as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), text.as_mut_ptr().cast(), &mut bytes) != 0 {
            return None;
        }
        let end = text.iter().position(|unit| *unit == 0).unwrap_or(text.len());
        Some(String::from_utf16_lossy(&text[..end]))
    }
}

/// `command` is somewhere programs are looked for.
pub fn on_path(command: &str) -> bool {
    found_in(command, &search_dirs(), cfg!(windows))
}

/// What this system needs and does not have.
pub fn missing() -> Vec<&'static Prerequisite> {
    let dirs = search_dirs();
    ALL.iter().filter(|p| p.here() && !found_in(p.command, &dirs, cfg!(windows))).collect()
}

impl Prerequisite {
    /// The updater installs it by itself on this system: on Windows through
    /// winget, when winget is there; on Unix when it has an installer of its
    /// own that asks for no password (Claude Code).
    pub fn installable(&self) -> bool {
        if cfg!(windows) { self.winget.is_some() && found_in("winget", &search_dirs(), true) } else { self.script.is_some() }
    }

    /// Installs it: through winget on Windows, by its own installer on Unix.
    pub fn install(&self, say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
        let nothing = || anyhow::anyhow!("{}: nothing to install it by here", self.command);
        if cfg!(windows) {
            let id = self.winget.ok_or_else(nothing)?;
            say(format!("installing {} ({id}), for {}…", self.command, self.purpose));
            crate::run(crate::tool("winget").args(winget_install(id)), say)?;
        } else {
            let script = self.script.ok_or_else(nothing)?;
            say(format!("installing {}, for {}…", self.command, self.purpose));
            crate::run(std::process::Command::new("bash").args(["-c", script]), say)?;
        }
        anyhow::ensure!(found_in(self.command, &search_dirs(), cfg!(windows)), "{} is still not found once installed", self.command);
        Ok(())
    }
}

/// winget's arguments to install `id`, asking nothing. `--source winget`:
/// the Store's source, asked too by default, fails where there is no Store,
/// and winget then installs nothing.
fn winget_install(id: &str) -> [&str; 10] {
    ["install", "--id", id, "--exact", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements", "--disable-interactivity"]
}

/// Makes what is missing be there where the updater can (winget on
/// Windows; on Unix what has an installer of its own, no password asked),
/// and says the rest, each with how to install it; answers the commands
/// still missing.
pub fn ensure(say: &mut dyn FnMut(String)) -> Vec<&'static str> {
    for p in missing().into_iter().filter(|p| p.installable()) {
        if let Err(error) = p.install(say) {
            say(format!("✗ {}: {error:#}", p.command));
        }
    }
    report(say)
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

// ── Suggested ────────────────────────────────────────────────────────────

/// Not needed, but worth having beside Terminal Velocity: said while it is
/// missing, installed only when asked (its **Install** button).
#[derive(Debug, PartialEq)]
pub struct Suggestion {
    /// The command looked for on the `PATH`.
    pub command: &'static str,
    /// What it gives, said to the user.
    pub purpose: &'static str,
    /// Its npm package: installed for the user alone, no password asked.
    package: &'static str,
}

/// The list, in the order they are said.
pub const SUGGESTED: &[Suggestion] = &[Suggestion {
    command: "ccusage",
    purpose: "Claude Code's tokens and cost by day, by project and by session",
    package: "ccusage",
}];

/// What is suggested and not on the machine.
pub fn suggested() -> Vec<&'static Suggestion> {
    let dirs = search_dirs();
    SUGGESTED.iter().filter(|s| !found_in(s.command, &dirs, cfg!(windows))).collect()
}

impl Suggestion {
    /// The command that installs it. On Unix into `~/.local` (its program in
    /// `~/.local/bin`), which needs no `sudo` whoever installed Node.js; on
    /// Windows npm's global folder is the user's already.
    pub fn how(&self, windows: bool) -> String {
        if windows { format!("npm install --global {}", self.package) } else { format!("npm install --global --prefix ~/.local {}", self.package) }
    }

    pub fn install(&self, say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
        say(format!("installing {}, for {}…", self.command, self.purpose));
        let mut npm = crate::tool("npm");
        npm.args(["install", "--global"]);
        if !cfg!(windows) {
            npm.arg("--prefix").arg(crate::bin_dir().parent().expect("~/.local"));
        }
        crate::run(npm.arg(self.package), say)?;
        anyhow::ensure!(found_in(self.command, &search_dirs(), cfg!(windows)), "{} is still not found once installed", self.command);
        Ok(())
    }
}

/// Says each suggestion missing, what it gives and how to install it.
pub fn suggest(say: &mut dyn FnMut(String)) {
    for s in suggested() {
        say(format!("· suggested, not installed: {}, for {}: {}", s.command, s.purpose, s.how(cfg!(windows))));
    }
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
    fn a_suggestion_installs_for_the_user_alone() {
        let ccusage = &SUGGESTED[0];
        assert_eq!(ccusage.how(false), "npm install --global --prefix ~/.local ccusage");
        assert_eq!(ccusage.how(true), "npm install --global ccusage");
    }

    #[test]
    fn winget_is_told_its_source_and_asks_nothing() {
        let args = winget_install("Git.Git");
        assert_eq!(&args[..3], ["install", "--id", "Git.Git"]);
        assert!(args.windows(2).any(|pair| pair == ["--source", "winget"]));
        assert!(args.contains(&"--silent") && args.contains(&"--disable-interactivity"));
    }

    #[test]
    #[cfg(windows)]
    fn the_registry_gives_the_path_a_new_session_would_have() {
        let dirs = registry_path();
        assert!(!dirs.is_empty(), "the machine's PATH is never empty");
        // Expanded: no %VARIABLE% left.
        assert!(dirs.iter().all(|dir| !dir.to_string_lossy().contains('%')), "{dirs:?}");
        assert!(search_path().to_string_lossy().to_lowercase().contains("system32"));
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
