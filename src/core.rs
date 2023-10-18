use std::{env, path::PathBuf};

use anyhow::{bail, Result};
use tmux_interface::{AttachSession, HasSession, NewSession, NewWindow, SwitchClient, Tmux};

pub fn in_tmux() -> bool {
    env::var("TMUX").is_ok()
}

pub fn open(path: &PathBuf) -> Result<()> {
    let session_name = session_name(path);

    let session_exists = Tmux::with_command(HasSession::new().target_session(session_name))
        .output()?
        .status()
        .success();

    if !session_exists {
        create_session(path)?;
    }

    // Attach to session
    if in_tmux() {
        Tmux::with_command(SwitchClient::new().target_session(session_name)).output()?;
    } else {
        Tmux::with_command(AttachSession::new().target_session(session_name)).output()?;
    }

    Ok(())
}

fn create_session(path: &PathBuf) -> Result<()> {
    let session_name = session_name(path);

    // Create new session
    // IDEA: Have this run in Lua
    let cmd = Tmux::new()
        .add_command(
            NewSession::new()
                .session_name(session_name)
                .shell_command("zsh -c nvim")
                .detached(),
        )
        .add_command(NewWindow::new().detached());
    let res = cmd.status()?;
    if !res.success() {
        bail!("tmux exited with non-zero status");
    }

    Ok(())
}

fn session_name(path: &PathBuf) -> &str {
    path.file_name()
        .map(|s| s.to_str())
        .flatten()
        .unwrap_or("unnamed")
}
