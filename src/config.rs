use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use home::home_dir;
use rlua::prelude::*;

const DEFAULT_CONFIG: &'static str = include_str!("../config.lua");

pub struct Config {
    lua: Lua,
}

impl Config {
    pub fn load() -> Result<Config> {
        let lua = Lua::new();

        lua.context(|ctx| {
            ctx.load(DEFAULT_CONFIG).exec()?;
            rlua::Result::Ok(())
        })
        .map_err(|e| anyhow!("Error loading default configuration:\n{}", e))?;

        let sources = Config::get_sources();
        for source in sources {
            let content = fs::read_to_string(&source);
            if let Ok(content) = content {
                lua.context(|ctx| {
                    ctx.load(&content).exec()?;
                    rlua::Result::Ok(())
                })
                .map_err(|e| anyhow!("Error loading configuration from {:?}:\n{}", source, e))?;
            }
        }

        Ok(Config { lua })
    }

    fn get_sources() -> Vec<PathBuf> {
        let options = [
            Some(PathBuf::from("/etc/tctrl/config.lua")),
            env::var("XDG_CONFIG_HOME")
                .ok()
                .map(|p| PathBuf::from(p).join("tctrl/config.lua"))
                .or_else(|| home_dir().map(|p| p.join(".config/tctrl/config.lua"))),
        ];

        let mut sources = Vec::new();
        for option in options.iter() {
            if let Some(path) = option {
                if path.exists() {
                    sources.push(path.clone());
                }
            }
        }

        sources
    }
}

impl Config {
    pub fn session_name(&self, path: &PathBuf) -> Result<String> {
        let session_name = self
            .lua
            .context(|ctx| {
                let globals = ctx.globals();

                let func: LuaFunction = globals.get("session_name")?;

                let param = ctx.create_table()?;
                param.set("path", path.to_string_lossy().to_string())?;
                param.set(
                    "filename",
                    path.file_name().unwrap().to_string_lossy().to_string(),
                )?;

                let res = func.call::<_, String>(param)?;

                rlua::Result::Ok(res)
            })
            .map_err(|e| anyhow!("Error getting session name:\n{}", e))?;

        Ok(session_name)
    }

    pub fn get_layout(&self, path: &PathBuf, session_name: &str) -> Result<Vec<String>> {
        let layouts = self.lua.context(|ctx| {
            let globals = ctx.globals();

            let func: LuaFunction = globals.get("get_layout")?;

            let param = ctx.create_table()?;
            param.set("path", path.to_string_lossy().to_string())?;
            param.set(
                "filename",
                path.file_name().unwrap().to_string_lossy().to_string(),
            )?;
            param.set("session_name", session_name)?;

            let res = func.call::<_, Vec<String>>(param)?;
            rlua::Result::Ok(res)
        })?;

        Ok(layouts)
    }

    pub fn list_projects(&self) -> Result<Vec<PathBuf>> {
        let folders = self
            .lua
            .context(|ctx| {
                let globals = ctx.globals();

                let func: LuaFunction = globals.get("list_projects")?;
                let res = func.call::<_, Vec<String>>(())?;

                rlua::Result::Ok(res)
            })
            .map_err(|e| anyhow!("Error getting session name:\n{}", e))?;

        let folders = folders
            .into_iter()
            .map(|s| PathBuf::from(s))
            .collect::<Vec<_>>();

        Ok(folders)
    }
}

pub fn get_default_config() -> &'static str {
    DEFAULT_CONFIG
}
