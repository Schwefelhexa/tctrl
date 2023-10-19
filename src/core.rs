use std::{env, path::PathBuf, process::Command};

use anyhow::{bail, Result};
use tmux_interface::{AttachSession, HasSession, NewSession, NewWindow, Tmux};

pub fn in_tmux() -> bool {
    env::var("TMUX").is_ok()
}

pub fn open(path: &PathBuf) -> Result<()> {
    let session_name = session_name(path);

    let session_exists = Tmux::with_command(HasSession::new().target_session(&session_name))
        .output()?
        .status()
        .success();

    if !session_exists {
        create_session(path)?;
    }

    // Attach to session
    if in_tmux() {
        // tmux_interface keeps the old session attached as well for some reason
        Command::new("tmux")
            .args(&["switch-client", "-t", session_name.as_str()])
            .output()?;
    } else {
        Tmux::with_command(AttachSession::new().target_session(&session_name)).output()?;
    }

    Ok(())
}

fn create_session(path: &PathBuf) -> Result<()> {
    let session_name = session_name(path);
    let path = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

    // Create new session
    // IDEA: Have this run in Lua
    let cmd = Tmux::new()
        .add_command(
            NewSession::new()
                .session_name(&session_name)
                .shell_command("zsh -c nvim")
                .start_directory(path)
                .detached(),
        )
        .add_command(NewWindow::new().detached().start_directory(path));
    let res = cmd.status()?;
    if !res.success() {
        bail!("tmux exited with non-zero status");
    }

    Ok(())
}

fn session_name(path: &PathBuf) -> String {
    // TODO: Configurable
    if path == &PathBuf::from("~") {
        return "base".to_owned();
    }

    path.file_name()
        .map(|s| s.to_str())
        .flatten()
        .map(|s| {
            s.to_string()
                .replace(" ", "_")
                .replace(",", "_")
                .replace(".", "_")
        })
        .unwrap_or("unnamed".to_owned())
}
