//! The multiplexer tvty lists, attaches and scrolls: tmux, or psmux on
//! Windows (its `tmux.exe` is an alias, not always installed). One place
//! says which program and which server, so that every call reaches the same
//! sessions.
//!
//! `TVTY_MUX_SERVER` names another server than the user's (`-L name`): a
//! test tvty sets it, and then cannot even see the user's own loops, let
//! alone attach one and resize it for its other clients. tmux also follows
//! `TMUX_TMPDIR`; psmux does not, `-L` is its only way.

use std::process::Command;

/// The multiplexer's program.
pub fn program() -> &'static str {
    if cfg!(windows) { "psmux" } else { "tmux" }
}

/// What goes before a command: the server, when not the user's.
pub fn server_args() -> Vec<String> {
    server_args_for(std::env::var("TVTY_MUX_SERVER").ok().as_deref())
}

fn server_args_for(server: Option<&str>) -> Vec<String> {
    match server.map(str::trim).filter(|s| !s.is_empty()) {
        Some(name) => vec!["-L".into(), name.into()],
        None => Vec::new(),
    }
}

/// `args` for the multiplexer, after the server's.
pub fn args(args: &[&str]) -> Vec<String> {
    server_args().into_iter().chain(args.iter().map(|a| a.to_string())).collect()
}

/// A multiplexer command, outside any session tvty itself runs in. On
/// Windows without a console window of its own: tvty is a window program,
/// and each command (the sessions are listed again and again) would flash
/// one.
pub fn command(command_args: &[&str]) -> Command {
    let mut command = Command::new(program());
    command.args(args(command_args)).env_remove("TMUX");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::server_args_for;

    #[test]
    fn another_server_only_when_named() {
        assert!(server_args_for(None).is_empty());
        assert!(server_args_for(Some("  ")).is_empty());
        assert_eq!(server_args_for(Some("tvty-test")), ["-L", "tvty-test"]);
    }
}
