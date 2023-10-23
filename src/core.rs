use std::{env, path::PathBuf, process::Command};

use anyhow::{bail, Result};
use tmux_interface::{AttachSession, HasSession, NewSession, NewWindow, Tmux};

use crate::config::Config;

pub fn in_tmux() -> bool {
    env::var("TMUX").is_ok()
}

pub fn open(path: &PathBuf, client: Option<&str>, config: &Config) -> Result<()> {
    let session_name = config.session_name(path)?;

    let session_exists = Tmux::with_command(HasSession::new().target_session(&session_name))
        .output()?
        .status()
        .success();

    if !session_exists {
        create_session(path, &session_name)?;
    }

    // Attach to session
    if in_tmux() || client.is_some() {
        // tmux_interface keeps the old session attached as well for some reason
        let mut args = vec!["switch-client", "-t", session_name.as_str()];
        if let Some(client) = client {
            args.push("-c");
            args.push(client);
        }
        Command::new("tmux").args(args).status()?;
    } else {
        Tmux::with_command(AttachSession::new().target_session(&session_name)).output()?;
    }

    Ok(())
}

fn create_session(path: &PathBuf, session_name: &str) -> Result<()> {
    let path = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

    // Create new session
    // IDEA: Have this run in Lua
    let cmd = Tmux::new()
        .add_command(
            NewSession::new()
                .session_name(session_name)
                .shell_command("zsh -c nvim")
                .start_directory(path)
                .detached(),
        )
        .add_command(NewWindow::new().target_window(session_name).detached().start_directory(path));
    let res = cmd.status()?;
    if !res.success() {
        bail!("tmux exited with non-zero status");
    }

    Ok(())
}

