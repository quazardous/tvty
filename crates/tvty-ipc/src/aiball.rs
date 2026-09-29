//! Where aiball is, and what to show it — found as aiball's own clients
//! find it (its `bin/launcher.js` and `src/client.ts`), so that tvty and the
//! `aiball` command never disagree. `docs/IPC.md`, "Where aiball is".

use std::fmt;
use std::path::{Path, PathBuf};

use crate::{Credentials, Endpoint, EndpointError};

/// aiball's port when nothing says otherwise.
pub const DEFAULT_PORT: u16 = 7777;

/// Where aiball is, and the credentials that go with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub endpoint: Endpoint,
    pub credentials: Credentials,
}

/// Why aiball cannot be located.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocateError {
    /// `AIBALL_SOCK` or `AIBALL_URL` is not an address.
    Endpoint(String, EndpointError),
    /// TCP, and no token: neither `AIBALL_TOKEN` nor one in `cli_env`.
    NoToken { endpoint: Endpoint, cli_env: PathBuf },
}

impl fmt::Display for LocateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Endpoint(var, error) => write!(f, "{var}: {error}"),
            Self::NoToken { endpoint, cli_env } => write!(
                f,
                "aiball is at {endpoint}, which needs a token: none in AIBALL_TOKEN, no machine-secret \
                 beside {} (a recent aiball writes it when it starts), and none in that file",
                cli_env.display()
            ),
        }
    }
}

impl std::error::Error for LocateError {}

/// What `locate` reads: the process's environment and files, or a test's.
pub trait Sources {
    fn var(&self, name: &str) -> Option<String>;
    fn home_dir(&self) -> Option<PathBuf>;
    fn is_socket(&self, path: &Path) -> bool;
    fn read(&self, path: &Path) -> Option<String>;
}

/// The running process's.
pub struct System;

impl Sources for System {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// aiball's daemon takes Node's `homedir()`: `%USERPROFILE%` on
    /// Windows, `$HOME` elsewhere.
    fn home_dir(&self) -> Option<PathBuf> {
        let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        std::env::var_os(var).filter(|v| !v.is_empty()).map(PathBuf::from)
    }

    fn is_socket(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            std::fs::metadata(path).is_ok_and(|m| m.file_type().is_socket())
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            false
        }
    }

    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// aiball's home: `$AIBALL_HOME`, else `~/.local/share/aiball`.
pub fn home(sources: &dyn Sources) -> PathBuf {
    match sources.var("AIBALL_HOME").filter(|v| !v.is_empty()) {
        Some(home) => PathBuf::from(home),
        None => sources.home_dir().unwrap_or_default().join(".local").join("share").join("aiball"),
    }
}

/// Where aiball is, for this process.
pub fn locate() -> Result<Location, LocateError> {
    locate_in(&System)
}

/// Where aiball is, from `sources`:
/// 1. `AIBALL_SOCK` when set — set but empty forces TCP;
/// 2. on Unix, `$AIBALL_HOME/sock` when it is a socket;
/// 3. TCP: `AIBALL_URL`, else `http://127.0.0.1:$AIBALL_PORT` (7777), with
///    `AIBALL_TOKEN`; else, on the loopback, `$AIBALL_HOME/machine-secret`
///    (aiball then treats tvty as it treats its socket's callers); else the
///    token in `$AIBALL_HOME/cli-env`.
pub fn locate_in(sources: &dyn Sources) -> Result<Location, LocateError> {
    let home = home(sources);
    match sources.var("AIBALL_SOCK") {
        Some(sock) if !sock.is_empty() => {
            let endpoint = Endpoint::parse(&sock).map_err(|e| LocateError::Endpoint("AIBALL_SOCK".into(), e))?;
            return Ok(Location { endpoint, credentials: Credentials::Peer });
        }
        Some(_) => {}
        None => {
            let sock = home.join("sock");
            if sources.is_socket(&sock) {
                return Ok(Location { endpoint: Endpoint::Unix(sock), credentials: Credentials::Peer });
            }
        }
    }
    let endpoint = match sources.var("AIBALL_URL").filter(|v| !v.is_empty()) {
        // A URL is the user's own choice: another host is allowed.
        Some(url) => Endpoint::parse_remote(&url).map_err(|e| LocateError::Endpoint("AIBALL_URL".into(), e))?,
        None => {
            let port = sources.var("AIBALL_PORT").and_then(|p| p.trim().parse().ok()).unwrap_or(DEFAULT_PORT);
            Endpoint::Tcp(([127, 0, 0, 1], port).into())
        }
    };
    let cli_env = home.join("cli-env");
    let loopback = matches!(endpoint, Endpoint::Tcp(addr) if addr.ip().is_loopback());
    let token = sources
        .var("AIBALL_TOKEN")
        .filter(|t| !t.trim().is_empty())
        // Never sent off this machine: it proves "same user here", nothing
        // anywhere else.
        .or_else(|| loopback.then(|| sources.read(&home.join("machine-secret")).and_then(|text| machine_secret(&text))).flatten())
        .or_else(|| sources.read(&cli_env).and_then(|text| env_file_var(&text, "AIBALL_TOKEN")));
    match token {
        Some(token) => Ok(Location { endpoint, credentials: Credentials::Bearer(token.trim().to_string()) }),
        None => Err(LocateError::NoToken { endpoint, cli_env }),
    }
}

/// aiball's machine secret, as its daemon writes it (`aiball-machine-` and
/// 64 hex digits, one line); anything else is not one.
fn machine_secret(text: &str) -> Option<String> {
    let secret = text.trim();
    let hex = secret.strip_prefix("aiball-machine-")?;
    (hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))).then(|| secret.to_string())
}

/// A variable of an env file, read as aiball's launcher reads `cli-env`:
/// `[export ]NAME=value` lines, `#` comments and blank lines skipped, one
/// layer of matching quotes stripped.
pub fn env_file_var(text: &str, name: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let line = line.strip_prefix("export").filter(|rest| rest.starts_with(char::is_whitespace)).map_or(line, str::trim_start);
        let (key, value) = line.split_once('=')?;
        let valid = key.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid || key != name {
            return None;
        }
        let value = value.trim();
        let unquoted = [('"', '"'), ('\'', '\'')]
            .iter()
            .find_map(|&(open, close)| value.strip_prefix(open)?.strip_suffix(close))
            .unwrap_or(value);
        Some(unquoted.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Fake {
        vars: HashMap<&'static str, &'static str>,
        sockets: Vec<PathBuf>,
        files: HashMap<PathBuf, &'static str>,
    }

    impl Sources for Fake {
        fn var(&self, name: &str) -> Option<String> {
            self.vars.get(name).map(|v| v.to_string())
        }
        fn home_dir(&self) -> Option<PathBuf> {
            Some(PathBuf::from("/srv/u"))
        }
        fn is_socket(&self, path: &Path) -> bool {
            self.sockets.iter().any(|s| s == path)
        }
        fn read(&self, path: &Path) -> Option<String> {
            self.files.get(path).map(|t| t.to_string())
        }
    }

    fn aiball_home() -> PathBuf {
        PathBuf::from("/srv/u").join(".local").join("share").join("aiball")
    }

    #[test]
    fn an_explicit_socket_wins() {
        let fake = Fake { vars: HashMap::from([("AIBALL_SOCK", "/tmp/a/sock"), ("AIBALL_TOKEN", "t")]), ..Default::default() };
        let at = locate_in(&fake).unwrap();
        assert_eq!(at.endpoint, Endpoint::Unix("/tmp/a/sock".into()));
        assert_eq!(at.credentials, Credentials::Peer);
    }

    #[test]
    fn the_home_socket_when_it_is_one() {
        let fake = Fake { sockets: vec![aiball_home().join("sock")], ..Default::default() };
        assert_eq!(locate_in(&fake).unwrap().endpoint, Endpoint::Unix(aiball_home().join("sock")));
    }

    #[test]
    fn an_empty_aiball_sock_forces_tcp() {
        let fake = Fake {
            vars: HashMap::from([("AIBALL_SOCK", ""), ("AIBALL_TOKEN", "t")]),
            sockets: vec![aiball_home().join("sock")],
            ..Default::default()
        };
        assert_eq!(locate_in(&fake).unwrap().endpoint, Endpoint::Tcp("127.0.0.1:7777".parse().unwrap()));
    }

    #[test]
    fn tcp_takes_the_port_and_the_token_from_cli_env() {
        let fake = Fake {
            vars: HashMap::from([("AIBALL_PORT", "7780")]),
            files: HashMap::from([(aiball_home().join("cli-env"), "# made by aiball\nexport AIBALL_TOKEN=aiball-abc\n")]),
            ..Default::default()
        };
        let at = locate_in(&fake).unwrap();
        assert_eq!(at.endpoint, Endpoint::Tcp("127.0.0.1:7780".parse().unwrap()));
        assert_eq!(at.credentials, Credentials::Bearer("aiball-abc".into()));
    }

    #[test]
    fn the_environment_token_wins_over_the_file_and_a_url_over_the_port() {
        let fake = Fake {
            vars: HashMap::from([("AIBALL_TOKEN", "from-env"), ("AIBALL_URL", "http://127.0.0.1:9000"), ("AIBALL_PORT", "7780")]),
            files: HashMap::from([(aiball_home().join("cli-env"), "export AIBALL_TOKEN=from-file\n")]),
            ..Default::default()
        };
        let at = locate_in(&fake).unwrap();
        assert_eq!(at.endpoint, Endpoint::Tcp("127.0.0.1:9000".parse().unwrap()));
        assert_eq!(at.credentials, Credentials::Bearer("from-env".into()));
    }

    #[test]
    fn tcp_without_a_token_says_where_it_looked() {
        match locate_in(&Fake::default()) {
            Err(LocateError::NoToken { cli_env, .. }) => assert_eq!(cli_env, aiball_home().join("cli-env")),
            other => panic!("{other:?}"),
        }
    }

    const SECRET: &str = "aiball-machine-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn the_machine_secret_wins_over_cli_env_on_the_loopback() {
        let fake = Fake {
            files: HashMap::from([
                (aiball_home().join("machine-secret"), "aiball-machine-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n"),
                (aiball_home().join("cli-env"), "export AIBALL_TOKEN=from-file\n"),
            ]),
            ..Default::default()
        };
        assert_eq!(locate_in(&fake).unwrap().credentials, Credentials::Bearer(SECRET.into()));
    }

    #[test]
    fn the_machine_secret_never_leaves_the_machine() {
        let fake = Fake {
            vars: HashMap::from([("AIBALL_URL", "http://10.0.0.2:7777")]),
            files: HashMap::from([(aiball_home().join("machine-secret"), SECRET)]),
            ..Default::default()
        };
        assert!(matches!(locate_in(&fake), Err(LocateError::NoToken { .. })));
    }

    #[test]
    fn the_environment_token_wins_over_the_machine_secret() {
        let fake = Fake {
            vars: HashMap::from([("AIBALL_TOKEN", "from-env")]),
            files: HashMap::from([(aiball_home().join("machine-secret"), SECRET)]),
            ..Default::default()
        };
        assert_eq!(locate_in(&fake).unwrap().credentials, Credentials::Bearer("from-env".into()));
    }

    #[test]
    fn a_malformed_machine_secret_is_not_one() {
        let fake = Fake {
            files: HashMap::from([(aiball_home().join("machine-secret"), "aiball-machine-short")]),
            ..Default::default()
        };
        assert!(matches!(locate_in(&fake), Err(LocateError::NoToken { .. })));
    }

    #[test]
    fn aiball_home_moves_everything() {
        let fake = Fake {
            vars: HashMap::from([("AIBALL_HOME", "/srv/aiball")]),
            files: HashMap::from([(PathBuf::from("/srv/aiball").join("cli-env"), "AIBALL_TOKEN='quoted'")]),
            ..Default::default()
        };
        assert_eq!(locate_in(&fake).unwrap().credentials, Credentials::Bearer("quoted".into()));
    }

    #[test]
    fn env_files_are_read_as_aiballs_launcher_reads_them() {
        let text = "# comment\n\nexport OTHER=1\nexport AIBALL_TOKEN=\"a b\"\nAIBALL_TOKEN=second\n";
        assert_eq!(env_file_var(text, "AIBALL_TOKEN").as_deref(), Some("a b"));
        assert_eq!(env_file_var("exportAIBALL_TOKEN=x", "AIBALL_TOKEN"), None);
        assert_eq!(env_file_var("AIBALL_TOKEN=plain", "AIBALL_TOKEN").as_deref(), Some("plain"));
        assert_eq!(env_file_var("1BAD=x\nAIBALL_TOKEN=ok", "AIBALL_TOKEN").as_deref(), Some("ok"));
    }
}
