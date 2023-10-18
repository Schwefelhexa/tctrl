use std::{env, path::PathBuf};

use anyhow::{bail, Result};
use tmux_interface::{HasSession, NewSession, NewWindow, SelectWindow, SwitchClient, Tmux, AttachSession};

pub fn in_tmux() -> bool {
    env::var("TMUX").is_ok()
}

pub fn open(path: &PathBuf) -> Result<()> {
    let session_name = path
        .file_name()
        .map(|s| s.to_str())
        .flatten()
        .unwrap_or("unnamed");

    Tmux::with_command(AttachSession::new()).output()?;

    let session_exists = Tmux::with_command(HasSession::new().target_session(session_name))
        .output()?
        .status()
        .success();

    // Attach to existing session, if possible
    if session_exists {
        Tmux::with_command(SwitchClient::new().target_session(session_name)).output()?;
        return Ok(());
    }

    // Create new session
    // IDEA: Have this run in Lua
    let cmd = Tmux::new()
        .add_command(
            NewSession::new()
                .session_name(session_name)
                .shell_command("zsh -c nvim"),
        )
        .add_command(NewWindow::new())
        .add_command(SelectWindow::new().target_window("1"));
    let res = cmd.status()?;
    if !res.success() {
        bail!("tmux exited with non-zero status");
    }
    Ok(())
}
