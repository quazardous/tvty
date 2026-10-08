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
    /// It is on the machine, but did not say its version when asked (its
    /// command failed): not the same as not installed.
    pub silent: bool,
    /// It runs from a development checkout (aiball's `install.mode: dev`):
    /// it does not update itself, its owner updates it by hand.
    pub dev: bool,
    /// The command that updates it by hand, as it says it.
    pub update_command: Option<String>,
}

/// Where a program stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Missing,
    /// There, but it did not answer when asked its version (half installed,
    /// or broken): said as such, and mended by installing it again.
    Silent,
    /// A development install (run from its checkout): updated by hand,
    /// never from here.
    Dev,
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
        let Some(installed) = self.installed.as_deref() else {
            return if self.silent { State::Silent } else { State::Missing };
        };
        // Whatever its version: its own update refuses to run.
        if self.dev {
            return State::Dev;
        }
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

// ── A development build ──────────────────────────────────────────────────

/// What a Terminal Velocity run from its checkout tells the updater it
/// opens: its executable. The updater has no release to give it: it says
/// how to update it from the checkout.
pub const DEV_EXE_VAR: &str = "TVTY_DEV_EXE";
/// And the version it runs.
pub const DEV_VERSION_VAR: &str = "TVTY_DEV_VERSION";

/// A Terminal Velocity built from its checkout, and run from there: under
/// the checkout's `target`, or a copy of its build kept there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevBuild {
    /// The checkout: its `Cargo.toml` is Terminal Velocity's.
    pub checkout: PathBuf,
    /// The executable that runs.
    pub running: PathBuf,
}

impl DevBuild {
    /// `exe`'s checkout, when it runs from one.
    pub fn of(exe: &Path) -> Option<Self> {
        let target = exe.ancestors().skip(1).find(|dir| dir.file_name().is_some_and(|name| name == "target"))?;
        let checkout = target.parent()?;
        let manifest = std::fs::read_to_string(checkout.join("Cargo.toml")).ok()?;
        let ours = manifest.lines().any(|line| line.split_whitespace().collect::<String>() == "name=\"tvty\"");
        ours.then(|| Self { checkout: checkout.to_path_buf(), running: exe.to_path_buf() })
    }

    /// The build cargo makes: the release profile's when it runs from it,
    /// the development one's otherwise.
    fn release(&self) -> bool {
        self.running.parent().and_then(Path::file_name).is_some_and(|name| name == "release")
    }

    fn built(&self) -> PathBuf {
        self.checkout.join("target").join(if self.release() { "release" } else { "debug" }).join(exe("tvty"))
    }

    /// The updater built from the same checkout: beside the executable
    /// when it is there, else cargo's.
    pub fn updater(&self) -> Option<PathBuf> {
        let beside = self.running.parent()?.join(exe("tvty-updater"));
        Some(if beside.exists() { beside } else { self.built().with_file_name(exe("tvty-updater")) })
    }

    /// Whether the build must then be copied where it runs from (a copy
    /// the shortcut starts, out of cargo's way).
    pub fn copied(&self) -> bool {
        self.built() != self.running
    }

    /// The line that updates it: the checkout pulled, built, and the build
    /// copied where it runs from when that is a copy.
    pub fn update_command(&self) -> String {
        let quote = |path: &Path| format!("'{}'", path.display());
        let profile = if self.release() { " --release" } else { "" };
        let mut steps = vec![
            format!("git -C {} pull --ff-only", quote(&self.checkout)),
            format!("cargo build{profile} --manifest-path {}", quote(&self.checkout.join("Cargo.toml"))),
        ];
        if self.copied() {
            steps.push(if cfg!(windows) {
                format!("Copy-Item {} {}", quote(&self.built()), quote(&self.running))
            } else {
                format!("cp {} {}", quote(&self.built()), quote(&self.running))
            });
        }
        steps.join(if cfg!(windows) { "; " } else { " && " })
    }
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
    match std::env::var("TVTY_SETUP_FROM").ok().filter(|from| !from.trim().is_empty()) {
        // A build tried before its release: its files are where this says,
        // not asked of GitHub.
        Some(from) => {
            say(format!("installing Terminal Velocity from {from}…"));
            run_release_installer("tvty", from.trim(), say)?;
        }
        None => {
            say("downloading and installing the latest release…".into());
            match updater.run_sync() {
                Ok(Some(done)) => say(format!("installed Terminal Velocity {}", done.new_version)),
                Ok(None) => say("Terminal Velocity is up to date".into()),
                // GitHub's API answers a few requests an hour from one
                // address, then refuses; the release's installer is also
                // behind a plain link, which it does not count.
                Err(error) => {
                    say(format!("the latest release could not be asked ({error}): its installer taken by its link"));
                    run_release_installer("tvty", RELEASE_LINKS, say)?;
                }
            }
        }
    }
    anyhow::ensure!(bin_dir().join(exe("tvty")).exists(), "Terminal Velocity is not in {} once installed", bin_dir().display());
    install_launcher(say)?;
    Ok(())
}

/// Where the latest release's files are, by their links.
const RELEASE_LINKS: &str = "https://github.com/quazardous/tvty/releases/latest/download";

/// Runs the release's own installer of `app` (dist's: a PowerShell script on
/// Windows, a shell script elsewhere), taken from `base`: a URL, or a folder
/// holding the release's files.
fn run_release_installer(app: &str, base: &str, say: &mut dyn FnMut(String)) -> anyhow::Result<()> {
    let base = base.trim_end_matches(['/', '\\']);
    let folder = Path::new(base).is_dir();
    let script = format!("{app}-installer.{}", if cfg!(windows) { "ps1" } else { "sh" });
    let mut command = if cfg!(windows) {
        let mut command = tool("powershell");
        command.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass"]);
        if folder {
            command.arg("-File").arg(Path::new(base).join(&script));
        } else {
            command.args(["-Command", &format!("irm '{base}/{script}' | iex")]);
        }
        command
    } else {
        let mut command = Command::new("sh");
        if folder {
            command.arg(Path::new(base).join(&script));
        } else {
            command.args(["-c", &format!("curl --proto '=https' --tlsv1.2 -LsSf '{base}/{script}' | sh")]);
        }
        command
    };
    if folder {
        // The installer takes its archive from the same folder.
        command.env(format!("{}_DOWNLOAD_URL", app.replace('-', "_").to_uppercase()), file_url(base));
    }
    run(&mut command, say)
}

/// A folder as a `file:` URL.
fn file_url(folder: &str) -> String {
    let path = folder.replace('\\', "/");
    format!("file://{}{path}", if path.starts_with('/') { "" } else { "/" })
}

/// Everything, without a window (`tvty-updater --install`, what the Windows
/// setup script runs): aiball installed when missing and updated when older
/// than Terminal Velocity needs, then Terminal Velocity installed or updated,
/// its launchers in place either way; what is suggested said last.
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
        State::Missing | State::TooOld | State::Silent if missing.iter().any(|m| matches!(*m, "git" | "node" | "pwsh")) => {
            Err(anyhow::anyhow!("not installed: it needs what is missing above"))
        }
        State::Missing => install_aiball(say),
        State::TooOld => update_aiball(say),
        State::Dev => {
            say("aiball is a development install (it runs from its checkout): left as it is, updated by hand".into());
            Ok(())
        }
        // Half installed, or broken: its installer run again mends it.
        State::Silent => {
            say("aiball is there, but its command does not answer: installing it again".into());
            install_aiball(say)
        }
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
        State::UpToDate | State::Unknown | State::Silent | State::Dev => {
            say("Terminal Velocity is installed".into());
            install_launcher(say)
        }
    };
    if let Err(error) = tvty {
        say(format!("✗ Terminal Velocity: {error:#}"));
        failed.push("Terminal Velocity");
    }
    // Worth having, never installed unasked: said, with how.
    prerequisites::suggest(say);
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
    // Its command is there and said no version: it failed (said why on its
    // error output), which is not "not installed".
    if status.installed.is_none() && (program("aiball").is_absolute() || prerequisites::on_path("aiball")) {
        status.silent = true;
        let why = String::from_utf8_lossy(&out.stderr).lines().map(str::trim).find(|line| !line.is_empty()).map(str::to_string);
        status.note = Some(format!("`aiball --json version` did not answer: {}", why.or(status.note.take()).unwrap_or_else(|| out.status.to_string())));
    } else if status.installed.is_none() {
        // Not there at all: nothing to explain.
        status.note = None;
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
    status.dev = v.pointer("/install/mode").and_then(serde_json::Value::as_str) == Some("dev");
    status.update_command = text(v.get("update_command")).filter(|_| status.dev);
}

// ── The log ──────────────────────────────────────────────────────────────

/// Where the updater keeps what it says: `updater.log`, beside Terminal
/// Velocity's own log (its state directory) — on Windows beside the setup
/// script's `setup.log`, one folder up (`%LOCALAPPDATA%\tvty`).
pub fn log_file() -> Option<PathBuf> {
    let state = tvty_config::dir("tvty", tvty_config::Place::State)?;
    let dir = if cfg!(windows) { state.parent()?.to_path_buf() } else { state };
    Some(dir.join("updater.log"))
}

/// A line as it may be pasted where anyone reads it (an issue): the colour
/// codes a program wrote taken out, the home folder written `~`.
pub fn plain(line: &str) -> String {
    plain_in(line, tvty_config::home().as_deref())
}

fn plain_in(line: &str, home: Option<&Path>) -> String {
    // Escape sequences: `ESC [ … letter` (colours, moves), `ESC ] … BEL`
    // (titles); any other escape goes alone.
    let mut text = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            text.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            Some(']') => {
                for c in chars.by_ref() {
                    if c == '\u{7}' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    let Some(home) = home.map(|home| home.to_string_lossy().to_string()).filter(|home| home.len() > 1) else { return text };
    // As written, and with the other slash (git and node write `/` on Windows).
    let other = if home.contains('\\') { home.replace('\\', "/") } else { home.clone() };
    text.replace(&home, "~").replace(&other, "~")
}

/// A line the updater said, kept in its log with when it was said (UTC),
/// [`plain`]. A log that cannot be written is no reason to stop.
pub fn log_append(line: &str) {
    let line = &plain(line);
    use std::io::Write as _;
    let Some(file) = log_file() else { return };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut log) = std::fs::OpenOptions::new().create(true).append(true).open(file) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default();
        let _ = writeln!(log, "{} {line}", utc(now));
    }
}

/// `2026-09-30 13:05:24`, from seconds since the epoch.
fn utc(seconds: u64) -> String {
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let (day, month) = (doy - (153 * mp + 2) / 5 + 1, if mp < 10 { mp + 3 } else { mp - 9 });
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}", rest / 3_600, rest % 3_600 / 60, rest % 60)
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
    use super::{DevBuild, State, Status, latest_tag, older, plain_in, read_aiball_version};
    use std::path::Path;

    #[test]
    fn a_line_is_kept_without_colours_nor_the_home_folder() {
        let home = Path::new("/srv/u");
        assert_eq!(plain_in("\u{1b}[32mdone\u{1b}[0m in /srv/u/.local/bin", Some(home)), "done in ~/.local/bin");
        assert_eq!(plain_in("\u{1b}]0;a title\u{7}said", Some(home)), "said");
        assert_eq!(plain_in("nothing to change", Some(home)), "nothing to change");
        // Windows' folder, written either way.
        let home = Path::new("C:\\U\\u");
        assert_eq!(plain_in("C:\\U\\u\\.local and C:/U/u/.local", Some(home)), "~\\.local and ~/.local");
        assert_eq!(plain_in("C:\\U\\u", None), "C:\\U\\u");
    }

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
    fn a_folder_is_a_file_url_on_either_system() {
        assert_eq!(super::file_url(r"C:\work\distrib"), "file:///C:/work/distrib");
        assert_eq!(super::file_url("/srv/distrib"), "file:///srv/distrib");
    }

    #[test]
    fn a_development_install_is_told_and_never_updated_from_here() {
        let v = serde_json::json!({"cli":"0.52.0","daemon":{"running":"0.52.0","installed":"0.52.0","latest":"0.53.0"},
            "install":{"mode":"dev","source":"/w/aiball","inferred":true},"update_command":"cd /w/aiball; git pull"});
        let mut status = Status::default();
        super::read_aiball_version(&v, &mut status);
        assert!(status.dev);
        assert_eq!(status.update_command.as_deref(), Some("cd /w/aiball; git pull"));
        // Not "update available", nor "too old": its own update would refuse.
        assert_eq!(status.state(None), State::Dev);
        assert_eq!(status.state(Some("0.60.0")), State::Dev);
        // An installed one says no such thing.
        let v = serde_json::json!({"cli":"0.52.0","daemon":{"installed":"0.52.0"},"install":{"mode":"installed"},"update_command":"aiball update"});
        let mut status = Status::default();
        super::read_aiball_version(&v, &mut status);
        assert!(!status.dev && status.update_command.is_none());
    }

    #[test]
    fn a_log_line_is_dated_in_utc() {
        assert_eq!(super::utc(0), "1970-01-01 00:00:00");
        assert_eq!(super::utc(1_790_773_524), "2026-09-30 13:05:24");
        assert_eq!(super::utc(951_782_400), "2000-02-29 00:00:00");
    }

    #[test]
    fn there_but_silent_is_not_missing() {
        let silent = Status { silent: true, ..Default::default() };
        assert_eq!(silent.state(None), State::Silent);
        // Once it says a version, it stands as any other.
        let answered = Status { installed: Some("0.5.0".into()), silent: true, ..Default::default() };
        assert_eq!(answered.state(None), State::Unknown);
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

    #[test]
    fn a_build_under_its_checkout_is_a_development_build() {
        let checkout = std::env::temp_dir().join(format!("tvty-devbuild-{}", std::process::id()));
        std::fs::create_dir_all(checkout.join("target").join("dev")).unwrap();
        std::fs::write(checkout.join("Cargo.toml"), "[package]
name = \"tvty\"
").unwrap();
        let running = checkout.join("target").join("dev").join(super::exe("tvty"));
        let dev = DevBuild::of(&running).expect("a development build");
        assert_eq!(dev.checkout, checkout);
        // A copy out of cargo's way: built, then copied there.
        assert!(dev.copied());
        let command = dev.update_command();
        assert!(command.starts_with(&format!("git -C '{}' pull --ff-only", checkout.display())), "{command}");
        assert!(command.contains("cargo build --manifest-path"), "{command}");
        assert!(command.ends_with(&format!("'{}'", running.display())), "{command}");
        // cargo's own output: nothing to copy.
        let built = checkout.join("target").join("debug").join(super::exe("tvty"));
        assert!(!DevBuild::of(&built).unwrap().copied());
        // Another project's checkout, or an installed one: none.
        std::fs::write(checkout.join("Cargo.toml"), "[package]
name = \"other\"
").unwrap();
        assert_eq!(DevBuild::of(&running), None);
        assert_eq!(DevBuild::of(Path::new("/usr/local/bin/tvty")), None);
        let _ = std::fs::remove_dir_all(&checkout);
    }

}
