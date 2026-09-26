//! aiball's daemon, when it is not running: tvty starts it, detached — it
//! serves the loops, the web UI and other machines too, so it never lives
//! and dies with tvty. Only the user's own aiball, at its usual socket, and
//! only through its systemd user service: a socket set by `AIBALL_SOCK` (a
//! test's throwaway aiball) is never started from here.

use std::process::{Command, Stdio};

/// What was found, and done.
#[derive(Debug, PartialEq)]
pub enum Start {
    /// It answers.
    Running,
    /// It did not; its service is starting.
    Started,
    /// It does not answer, and tvty has nothing to start it with.
    Missing(String),
}

/// Whether aiball answers, and if not, starts it when tvty may.
pub fn ensure() -> Start {
    let socket = crate::aiball::socket_path();
    if answers(&socket) {
        return Start::Running;
    }
    let chosen = std::env::var("AIBALL_SOCK").is_ok_and(|s| !s.is_empty());
    match decide(chosen, has_service()) {
        Some(()) => match start_service() {
            Ok(()) => Start::Started,
            Err(error) => Start::Missing(format!("aiball's service did not start: {error}")),
        },
        None if chosen => Start::Missing(format!("aiball does not answer at {}", socket.display())),
        None => Start::Missing("aiball is not running, and there is no aiball service to start".into()),
    }
}

/// Starts only the user's own aiball (no `AIBALL_SOCK`), and only through
/// its service.
fn decide(socket_chosen: bool, service: bool) -> Option<()> {
    (!socket_chosen && service).then_some(())
}

#[cfg(unix)]
fn answers(socket: &std::path::Path) -> bool {
    std::os::unix::net::UnixStream::connect(socket).is_ok()
}

#[cfg(not(unix))]
fn answers(_: &std::path::Path) -> bool {
    true
}

fn has_service() -> bool {
    Command::new("systemctl")
        .args(["--user", "cat", "aiball.service"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn start_service() -> anyhow::Result<()> {
    let status = Command::new("systemctl")
        .args(["--user", "start", "--no-block", "aiball.service"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    anyhow::ensure!(status.success(), "systemctl said {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::decide;

    #[test]
    fn only_the_users_own_aiball_is_started_and_only_by_its_service() {
        assert_eq!(decide(false, true), Some(()));
        assert_eq!(decide(false, false), None);
        // A test's throwaway aiball, or any socket chosen: never.
        assert_eq!(decide(true, true), None);
    }
}
