use crate::app::default_effect_manager;
use crate::app::App;
use crate::config::Config;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};

pub struct Persistence;

impl Persistence {
    fn get_data_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "pabloagn", "Kronos")
            .ok_or_else(|| anyhow::anyhow!("Could not find a valid home directory."))?;
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;
        Ok(data_dir.join("state.json"))
    }

    pub fn save(app: &App) -> Result<()> {
        let path = Self::get_data_path()?;
        let json = serde_json::to_string_pretty(app)
            .with_context(|| "Failed to serialize application state")?;
        fs::write(&path, json).with_context(|| format!("Failed to write state to {:?}", path))?;
        Ok(())
    }

    pub fn load(config: &Config) -> Result<Option<App>> {
        let path = Self::get_data_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read state from {:?}", path))?;
        if json.is_empty() {
            return Ok(None);
        }
        let mut app: App = serde_json::from_str(&json)
            .with_context(|| format!("Failed to deserialize state from {:?}", path))?;
        app.config = config.clone();
        app.effect_manager = default_effect_manager(); // Re-initialize non-deserialized fields
        Ok(Some(app))
    }
}
