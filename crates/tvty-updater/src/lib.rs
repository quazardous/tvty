//! Terminal Velocity Updater's engine, apart from its window: what is
//! installed of Terminal Velocity and aiball, what their latest releases
//! are, and the gestures that install, update or roll one back.
//!
//! Nothing is copied from either project's own tooling: Terminal Velocity
//! updates through its release's installer and install receipt (dist, read
//! by axoupdater); aiball through its own commands (`aiball version`,
//! `aiball update`) and, when it is missing, its own `install.sh`.
//!
//! Every gesture writes what it does, line by line, to the `say` it is
//! given: the window shows it as it goes.

pub mod prerequisites;

use std::io::{BufRead as _, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context as _, bail};
use axoupdater::{AxoUpdater, ReleaseSource, ReleaseSourceType, UpdateRequest, Version};

/// The oldest aiball this Terminal Velocity works with.
pub const MIN_AIBALL: &str = "0.49.0";

const OWNER: &str = "quazardous";
const TVTY_REPO: &str = "tvty";
const AIBALL_REPO: &str = "https://github.com/quazardous/aiball";

/// What is known of one program.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    /// The version installed, when it is.
    pub installed: Option<String>,
    /// The version running now, when it runs (aiball's daemon).
    pub running: Option<String>,
    /// The latest release, when it could be asked.
    pub latest: Option<String>,
    /// Why something is not known (no network, a command that failed).
    pub note: Option<String>,
}

/// Where a program stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Missing,
    /// Older than Terminal Velocity needs.
    TooOld,
    UpdateAvailable,
    UpToDate,
    /// Installed, its latest release not known.
    Unknown,
}

impl Status {
    /// Where it stands, `min` the oldest version that will do.
    pub fn state(&self, min: Option<&str>) -> State {
        let Some(installed) = self.installed.as_deref() else { return State::Missing };
        if min.is_some_and(|min| older(installed, min)) {
            return State::TooOld;
        }
        match self.latest.as_deref() {
            Some(latest) if older(installed, latest) => State::UpdateAvailable,
            Some(_) => State::UpToDate,
            None => State::Unknown,
        }
    }
}

/// `a` comes before `b`, as releases go ("0.4.0-beta.1" before "0.4.0").
pub fn older(a: &str, b: &str) -> bool {
    match (Version::parse(a.trim_start_matches('v')), Version::parse(b.trim_start_matches('v'))) {
        (Ok(a), Ok(b)) => a < b,
        _ => false,
    }
}

fn home() -> PathBuf {
    tvty_config::home().unwrap_or_default()
}

/// Where the installers put the programs (`~/.local/bin`, on Windows too).
pub fn bin_dir() -> PathBuf {
    home().join(".local").join("bin")
}

/// `name`'s file: `name.exe` on Windows.
fn exe(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

/// A program in `bin_dir`, else as the PATH finds it.
pub fn program(name: &str) -> PathBuf {
    let local = bin_dir().join(exe(name));
    if local.exists() { local } else { PathBuf::from(name) }
}

// ── Terminal Velocity ────────────────────────────────────────────────────

fn tvty_source() -> ReleaseSource {
    ReleaseSource { release_type: ReleaseSourceType::GitHub, owner: OWNER.into(), name: TVTY_REPO.into(), app_name: "tvty".into() }
}

/// Terminal Velocity's installed version (`tvty --version`) and its latest
/// release (GitHub, through axoupdater).
pub fn tvty_status() -> Status {
    let mut status = Status::default();
    match Command::new(program("tvty")).arg("--version").output() {
        Ok(out) if out.status.success() => {
            let said = String::from_utf8_lossy(&out.stdout);
            status.installed = said.split_whitespace().nth(1).map(str::to_string);
        }
        _ => {}
    }
    let mut updater = AxoUpdater::new_for("tvty");
    updater.set_release_source(tvty_source());
    // Terminal Velocity is in beta: its pre-releases are its releases.
    updater.configure_version_specifier(UpdateRequest::LatestMaybePrerelease);
    let latest = runtime().block_on(async { updater.query_new_version().await.map(|v| v.map(|v| v.to_string())) });
    match latest {
        Ok(latest) => status.latest = latest,
        Err(error) => status.note = Some(said_of(&error.to_string())),
    }
    status
}

/// A release query's failure, in words: a repository with no release yet
/// says 404.
fn said_of(error: &str) -> String {
    if error.contains("404") || error.contains("no stable releases") || error.contains("no releases") {
        "no release published yet".into()
    } else {
        format!("latest release not known: {error}")
    }
}

/// Installs or updates Terminal Velocity from its latest release, the one
/// in place kept aside to go back to; then its launcher and icon.
pub fn update_tvty(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    let installed = bin_dir().join(exe("tvty"));
    if installed.exists() {
        let kept = previous_dir().join(exe("tvty"));
        std::fs::create_dir_all(previous_dir())?;
        std::fs::copy(&installed, &kept).with_context(|| format!("keeping {}", installed.display()))?;
        say(format!("kept the version in place, to go back to: {}", kept.display()));
    }
    let mut updater = AxoUpdater::new_for("tvty");
    // Its receipt says where it is and which version (an install through
    // the release's installer); without one, the release source and
    // ~/.local/bin, as that installer would.
    if updater.load_receipt().is_err() {
        say("no install receipt: installing from the latest release".into());
        updater.set_release_source(tvty_source());
        updater.set_install_dir(bin_dir().to_string_lossy().to_string());
        updater.set_current_version(Version::parse("0.0.0").expect("a version"))?;
    }
    updater.configure_version_specifier(UpdateRequest::LatestMaybePrerelease);
    updater.disable_installer_output();
    say("downloading and installing the latest release…".into());
    match updater.run_sync()? {
        Some(done) => say(format!("installed Terminal Velocity {}", done.new_version)),
        None => say("Terminal Velocity is up to date".into()),
    }
    install_launcher(say)?;
    Ok(())
}

/// Everything, without a window (`tvty-updater --install`, what the Windows
/// setup script runs): aiball installed when missing and updated when older
/// than Terminal Velocity needs, then Terminal Velocity installed or updated,
/// its launchers in place either way.
pub fn install_all(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    // Each part tried whatever the others did: what failed is said, and
    // the rest is in place.
    let mut failed = Vec::new();
    // What is missing is said first, with how to get it: without Claude
    // Code, say, everything installs and no loop ever starts.
    // Installed here when it can be without a password (Claude Code).
    let missing = prerequisites::ensure(say);
    failed.extend(missing.iter().copied());
    // aiball's install clones it and runs on Node.js: not tried without.
    let aiball = match aiball_status().state(Some(MIN_AIBALL)) {
        State::Missing | State::TooOld if missing.iter().any(|m| matches!(*m, "git" | "node" | "pwsh")) => {
            Err(anyhow::anyhow!("not installed: it needs what is missing above"))
        }
        State::Missing => install_aiball(say),
        State::TooOld => update_aiball(say),
        _ => {
            say("aiball is installed".into());
            Ok(())
        }
    };
    // Installed is not running: its daemon must answer, or tvty has
    // nothing to talk to.
    let aiball = aiball.and_then(|()| aiball_answers(say));
    if let Err(error) = aiball {
        say(format!("✗ aiball: {error:#}"));
        failed.push("aiball");
    }
    let tvty = match tvty_status().state(None) {
        State::Missing | State::TooOld | State::UpdateAvailable => update_tvty(say),
        // Installed, its latest release not asked or not known: kept.
        State::UpToDate | State::Unknown => {
            say("Terminal Velocity is installed".into());
            install_launcher(say)
        }
    };
    if let Err(error) = tvty {
        say(format!("✗ Terminal Velocity: {error:#}"));
        failed.push("Terminal Velocity");
    }
    anyhow::ensure!(failed.is_empty(), "not in place: {}", failed.join(", "));
    Ok(())
}

/// aiball's daemon answers: waited for a little, as its install starts it.
fn aiball_answers(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    for _ in 0..10 {
        if let Some(version) = aiball_status().running {
            say(format!("aiball's daemon answers ({version})"));
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    bail!("it is installed, but its daemon does not answer (`aiball --json version` says why; its log is named by its installer)")
}

/// Puts back the version kept by the last update.
pub fn rollback_tvty(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    let kept = previous_dir().join(exe("tvty"));
    if !kept.exists() {
        bail!("no earlier version kept");
    }
    std::fs::copy(&kept, bin_dir().join(exe("tvty")))?;
    say("the earlier version is back".into());
    Ok(())
}

/// Where the version replaced by the last update is kept.
pub fn previous_dir() -> PathBuf {
    home().join(".local").join("lib").join("tvty").join("previous")
}

/// An earlier version is kept, to go back to.
pub fn has_previous() -> bool {
    previous_dir().join(exe("tvty")).exists()
}

const DESKTOP: &str = include_str!("../../../packaging/linux/tvty.desktop");
/// Terminal Velocity's icon, shown by the updater too.
pub const ICON: &[u8] = include_bytes!("../../../assets/tvty.svg");
const UPDATER_DESKTOP: &str = include_str!("../packaging/tvty-updater.desktop");

/// Terminal Velocity's launcher and icon, and the updater's own, where the
/// desktop finds them: the Start menu on Windows, the applications' folder
/// elsewhere.
pub fn install_launcher(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    if cfg!(windows) {
        let programs = PathBuf::from(std::env::var_os("APPDATA").context("no APPDATA")?).join(r"Microsoft\Windows\Start Menu\Programs");
        return shortcuts_in(&programs, &bin_dir(), say);
    }
    let data = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".local/share"));
    launchers_in(&data, &bin_dir(), say)
}

/// The Start menu's shortcuts, made by PowerShell (a .lnk is a COM
/// object's file); their icon is the one in each .exe.
fn shortcuts_in(programs: &Path, bin: &Path, say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    let script = shortcuts_script(programs, bin);
    let out = tool("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).output().context("powershell")?;
    if !out.status.success() {
        bail!("the Start menu's shortcuts: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    say(format!("Terminal Velocity and its updater are in the Start menu ({})", programs.display()));
    Ok(())
}

fn shortcuts_script(programs: &Path, bin: &Path) -> String {
    // PowerShell's single quotes take everything literally but a quote, doubled.
    let quoted = |text: &str| format!("'{}'", text.replace('\'', "''"));
    let path = |p: &Path| quoted(&p.display().to_string());
    let mut script = String::from("$shell = New-Object -ComObject WScript.Shell\n");
    for (name, program, what) in [
        ("Terminal Velocity", "tvty.exe", "Your AI agents' terminals, grouped by project, with their tickets"),
        ("Terminal Velocity Updater", "tvty-updater.exe", "Install and update Terminal Velocity and aiball"),
    ] {
        script.push_str(&format!(
            "$l = $shell.CreateShortcut({}); $l.TargetPath = {}; $l.WorkingDirectory = {}; $l.Description = {}; $l.Save()\n",
            path(&programs.join(format!("{name}.lnk"))),
            path(&bin.join(program)),
            path(&home()),
            quoted(what),
        ));
    }
    script
}

fn launchers_in(data: &Path, bin: &Path, say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    let apps = data.join("applications");
    let icons = data.join("icons/hicolor/scalable/apps");
    std::fs::create_dir_all(&apps)?;
    std::fs::create_dir_all(&icons)?;
    let exec = |name: &str| bin.join(name).display().to_string();
    std::fs::write(apps.join("tvty.desktop"), DESKTOP.replace("@EXEC@", &exec("tvty")))?;
    std::fs::write(apps.join("tvty-updater.desktop"), UPDATER_DESKTOP.replace("@EXEC@", &exec("tvty-updater")))?;
    std::fs::write(icons.join("tvty.svg"), ICON)?;
    // The desktops read them at once when told (GNOME, KDE); a missing tool
    // only delays it to the next login.
    let _ = Command::new("update-desktop-database").arg(&apps).output();
    let _ = Command::new("gtk-update-icon-cache").args(["-f", "-t"]).arg(data.join("icons/hicolor")).output();
    say(format!("Terminal Velocity and its updater are in your applications ({})", apps.display()));
    Ok(())
}

/// Starts Terminal Velocity, on its own (it outlives the updater).
pub fn launch_tvty() -> anyhow::Result<()> {
    Command::new(program("tvty")).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().context("starting tvty")?;
    Ok(())
}

/// Where each project lives.
pub const TVTY_URL: &str = "https://github.com/quazardous/tvty";
pub const AIBALL_URL: &str = AIBALL_REPO;

// ── aiball ───────────────────────────────────────────────────────────────

/// aiball's versions, as `aiball --json version` says them: the command's,
/// the daemon's running, and the latest release.
pub fn aiball_status() -> Status {
    let mut status = Status::default();
    let out = match aiball().args(["--json", "version"]).output() {
        Ok(out) => out,
        Err(_) => return status,
    };
    match serde_json::from_slice::<serde_json::Value>(&out.stdout) {
        Ok(v) => read_aiball_version(&v, &mut status),
        Err(error) => status.note = Some(format!("aiball version: {error}")),
    }
    status
}

fn read_aiball_version(v: &serde_json::Value, status: &mut Status) {
    let text = |value: Option<&serde_json::Value>| value.and_then(serde_json::Value::as_str).map(str::to_string);
    let daemon = v.get("daemon");
    status.installed = text(daemon.and_then(|d| d.get("installed"))).or_else(|| text(v.get("cli")));
    status.running = text(daemon.and_then(|d| d.get("running")));
    status.latest = text(daemon.and_then(|d| d.get("latest")));
    status.note = text(daemon.and_then(|d| d.get("error"))).or_else(|| text(v.get("daemon_error")));
}

/// Updates aiball its own way (`aiball update`).
pub fn update_aiball(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    run(aiball().arg("update"), say)
}

/// The `aiball` command: on Windows a `.cmd`, which only `cmd` runs.
fn aiball() -> Command {
    let aiball = program("aiball");
    if cfg!(windows) && aiball.extension().is_none() {
        let mut command = tool("cmd");
        command.args(["/c", "aiball"]);
        command
    } else {
        Command::new(aiball)
    }
}

/// Installs aiball: its latest tag cloned, then its own installer
/// (`install.sh`, or `install.ps1` on Windows).
pub fn install_aiball(say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    // Its latest release; `TVTY_AIBALL_REF` names another tag or a branch
    // (to try a fix of aiball before its release).
    let tag = match std::env::var("TVTY_AIBALL_REF").ok().filter(|r| !r.trim().is_empty()) {
        Some(asked) => asked.trim().to_string(),
        None => latest_aiball_tag()?,
    };
    let dir = home().join(".local/src/aiball");
    if dir.exists() {
        say(format!("{} is there already: updating it to {tag}", dir.display()));
        run(tool("git").args(["fetch", "--depth", "1", "origin", &tag]).current_dir(&dir), say)?;
        run(tool("git").args(["-c", "advice.detachedHead=false", "checkout", "FETCH_HEAD"]).current_dir(&dir), say)?;
    } else {
        std::fs::create_dir_all(dir.parent().expect("a parent"))?;
        run(tool("git").args(["-c", "advice.detachedHead=false", "clone", "--depth", "1", "--branch", &tag, AIBALL_REPO]).arg(&dir), say)?;
    }
    if cfg!(windows) {
        // PowerShell 7 (`pwsh`), as aiball's Windows install says.
        run(tool("pwsh").args(["-NoProfile", "-File", "install.ps1"]).current_dir(&dir), say)
    } else {
        run(Command::new("bash").arg("./install.sh").current_dir(&dir), say)
    }
}

fn latest_aiball_tag() -> anyhow::Result<String> {
    let out = tool("git")
        .args(["ls-remote", "--tags", "--refs", "--sort=-v:refname", AIBALL_REPO, "v*"])
        .output()
        .context("git, to find aiball's releases")?;
    let said = String::from_utf8_lossy(&out.stdout);
    latest_tag(&said).context("no release tag of aiball found")
}

/// The newest stable release tag in `git ls-remote --tags` output.
fn latest_tag(listing: &str) -> Option<String> {
    let mut tags: Vec<&str> = listing
        .lines()
        .filter_map(|l| l.split('\t').nth(1)?.strip_prefix("refs/tags/"))
        .filter(|t| !t.contains('-') && Version::parse(t.trim_start_matches('v')).is_ok())
        .collect();
    tags.sort_by(|a, b| {
        let v = |t: &str| Version::parse(t.trim_start_matches('v')).ok();
        v(a).cmp(&v(b))
    });
    tags.last().map(|t| t.to_string())
}

// ── Running ──────────────────────────────────────────────────────────────

/// A program of the machine, as the updater runs them. On Windows it is
/// found along the `PATH` a new session would have
/// ([`prerequisites::search_path`]) and runs with it — what was installed a
/// moment ago is found, and finds its own tools — and without a console
/// window of its own: the updater is a window program, and each of them
/// would flash one.
pub(crate) fn tool(name: &str) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let path = prerequisites::search_path();
        // Found here, not by the process's own PATH, which may be stale.
        let program = std::env::split_paths(&path)
            .flat_map(|dir| ["exe", "cmd", "bat"].map(|ext| dir.join(name).with_extension(ext)))
            .find(|file| file.is_file() || file.symlink_metadata().is_ok_and(|entry| !entry.is_dir()));
        let mut command = Command::new(program.unwrap_or_else(|| PathBuf::from(name)));
        command.env("PATH", path).creation_flags(CREATE_NO_WINDOW);
        command
    }
    #[cfg(not(windows))]
    Command::new(name)
}

/// Runs `command`, each line it writes (out and err) said as it comes.
pub(crate) fn run(command: &mut Command, say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    say(format!("$ {}", shown(command)));
    let mut child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().with_context(|| shown(command))?;
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let err = child.stderr.take().map(|e| {
        let tx = tx.clone();
        std::thread::spawn(move || BufReader::new(e).lines().map_while(Result::ok).for_each(|l| drop(tx.send(l))))
    });
    let out = child.stdout.take().map(|o| std::thread::spawn(move || BufReader::new(o).lines().map_while(Result::ok).for_each(|l| drop(tx.send(l)))));
    for line in rx {
        say(line);
    }
    err.map(|t| t.join());
    out.map(|t| t.join());
    let status = child.wait()?;
    if !status.success() {
        bail!("{} ended with {status}", shown(command));
    }
    Ok(())
}

fn shown(command: &Command) -> String {
    let program = Path::new(command.get_program()).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    std::iter::once(program).chain(command.get_args().map(|a| a.to_string_lossy().to_string())).collect::<Vec<_>>().join(" ")
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a tokio runtime")
}

#[cfg(test)]
mod tests {
    use super::{State, Status, latest_tag, older, read_aiball_version};

    #[test]
    fn versions_order_as_releases() {
        assert!(older("0.4.0-beta.1", "0.4.0"));
        assert!(older("0.4.0", "0.10.0"));
        assert!(older("v0.48.2", "0.49.0"));
        assert!(!older("0.49.0", "0.49.0"));
        assert!(!older("junk", "0.1.0"));
    }

    #[test]
    fn a_status_says_where_it_stands() {
        let s = |installed: Option<&str>, latest: Option<&str>| Status {
            installed: installed.map(str::to_string),
            latest: latest.map(str::to_string),
            ..Default::default()
        };
        assert_eq!(s(None, Some("1.0.0")).state(None), State::Missing);
        assert_eq!(s(Some("0.48.0"), Some("0.49.0")).state(Some("0.49.0")), State::TooOld);
        assert_eq!(s(Some("0.4.0"), Some("0.5.0")).state(None), State::UpdateAvailable);
        assert_eq!(s(Some("0.5.0"), Some("0.5.0")).state(None), State::UpToDate);
        assert_eq!(s(Some("0.5.0"), None).state(None), State::Unknown);
    }

    #[test]
    fn aiball_says_its_versions() {
        let v = serde_json::json!({"cli":"0.49.0","daemon":{"running":"0.49.0","installed":"0.49.0","latest":"0.50.0","error":null}});
        let mut status = Status::default();
        read_aiball_version(&v, &mut status);
        assert_eq!(status.installed.as_deref(), Some("0.49.0"));
        assert_eq!(status.latest.as_deref(), Some("0.50.0"));
        assert_eq!(status.state(Some("0.49.0")), State::UpdateAvailable);
    }

    #[test]
    #[cfg(not(windows))]
    fn launchers_point_at_the_installed_programs() {
        let dir = std::env::temp_dir().join(format!("tvty-updater-launchers-{}", std::process::id()));
        let bin = std::path::Path::new("/opt/x/bin");
        super::launchers_in(&dir, bin, &mut |_| {}).unwrap();
        let tvty = std::fs::read_to_string(dir.join("applications/tvty.desktop")).unwrap();
        let updater = std::fs::read_to_string(dir.join("applications/tvty-updater.desktop")).unwrap();
        assert!(tvty.contains("Exec=/opt/x/bin/tvty\n") && updater.contains("Exec=/opt/x/bin/tvty-updater\n"));
        assert!(dir.join("icons/hicolor/scalable/apps/tvty.svg").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_start_menu_shortcuts_point_at_the_installed_programs() {
        let script = super::shortcuts_script(std::path::Path::new("P"), std::path::Path::new("B"));
        let target = |p: &str| std::path::Path::new("B").join(p).display().to_string();
        assert!(script.contains(&format!("TargetPath = '{}'", target("tvty.exe"))));
        assert!(script.contains(&format!("TargetPath = '{}'", target("tvty-updater.exe"))));
        // A quote is doubled, not the end of the string.
        assert!(script.contains("'Your AI agents'' terminals"));
    }

    #[test]
    #[cfg(windows)]
    fn the_start_menu_shortcuts_are_made() {
        let dir = std::env::temp_dir().join(format!("tvty-updater-shortcuts-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        super::shortcuts_in(&dir, std::path::Path::new(r"C:\tvty-bin"), &mut |_| {}).unwrap();
        assert!(dir.join("Terminal Velocity.lnk").exists());
        assert!(dir.join("Terminal Velocity Updater.lnk").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_newest_stable_tag() {
        let listing = "a\trefs/tags/v0.9.0\nb\trefs/tags/v0.10.0\nc\trefs/tags/v0.11.0-beta.1\nd\trefs/tags/junk\n";
        assert_eq!(latest_tag(listing).as_deref(), Some("v0.10.0"));
        assert_eq!(latest_tag(""), None);
    }
}
