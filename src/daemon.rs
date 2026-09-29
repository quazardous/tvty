//! aiball's daemon, when it is not running: tvty starts it, detached — it
//! serves the loops, the web UI and other machines too, so it never lives
//! and dies with tvty. Only the user's own aiball, where it usually is, and
//! only through its systemd user service: an aiball chosen by `AIBALL_SOCK`
//! or `AIBALL_URL` (a test's throwaway aiball) is never started from here.

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
    // Where aiball is may not be known yet: on a first start, before the
    // daemon ever ran, there is neither its socket nor its machine secret.
    // That is no reason not to start it.
    let at = crate::aiball::location();
    if at.as_ref().is_ok_and(|at| at.endpoint.connect().is_ok()) {
        return Start::Running;
    }
    let not_there = match &at {
        Ok(at) => format!("aiball does not answer at {}", at.endpoint),
        Err(error) => error.to_string(),
    };
    let chosen = ["AIBALL_SOCK", "AIBALL_URL"].iter().any(|v| std::env::var(v).is_ok_and(|s| !s.is_empty()));
    match decide(chosen, has_service()) {
        Some(()) => match start_service() {
            Ok(()) => Start::Started,
            Err(error) => Start::Missing(format!("aiball's service did not start: {error}")),
        },
        None if chosen => Start::Missing(not_there),
        None => Start::Missing(format!("{not_there}; there is no aiball service to start")),
    }
}

/// Starts only the user's own aiball (no `AIBALL_SOCK`/`AIBALL_URL`), and
/// only through its service.
fn decide(chosen: bool, service: bool) -> Option<()> {
    (!chosen && service).then_some(())
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
        // A test's throwaway aiball, or any address chosen: never.
        assert_eq!(decide(true, true), None);
    }
}
