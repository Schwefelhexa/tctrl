use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use home::home_dir;
use rlua::prelude::*;

pub struct Config {
    lua: Lua,
}

impl Config {
    pub fn load() -> Result<Config> {
        let lua = Lua::new();

        let path = home_dir()
            .ok_or_else(|| anyhow!("No homedir found"))?
            .join(".config/tctrl/config.lua");
        let source = fs::read_to_string(&path);
        if let Ok(source) = source {
            lua.context(|ctx| {
                ctx.load(&source).exec()?;

                rlua::Result::Ok(())
            })
            .map_err(|e| anyhow!("Error loading config.lua:\n{}", e))?;
        }

        Ok(Config { lua })
    }
}

impl Config {
    pub fn session_name(&self, path: &PathBuf) -> Result<String> {
        let session_name = self
            .lua
            .context(|ctx| {
                let globals = ctx.globals();

                let func: rlua::Result<LuaFunction> = globals.get("session_name");
                if let Err(_) = func {
                    return rlua::Result::Ok(None);
                }
                let func = func.unwrap();

                let param = ctx.create_table()?;
                param.set("path", path.to_string_lossy().to_string())?;
                param.set(
                    "filename",
                    path.file_name().unwrap().to_string_lossy().to_string(),
                )?;

                let res = func.call::<_, String>(param)?;

                rlua::Result::Ok(Some(res))
            })
            .map_err(|e| anyhow!("Error getting session name:\n{}", e))?
            .unwrap_or_else(|| self.session_name_default(path));

        Ok(session_name)
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

    fn session_name_default(&self, path: &PathBuf) -> String {
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
}
