use std::env;

pub fn in_tmux() -> bool {
    env::var("TMUX").is_ok()
}

