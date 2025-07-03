use crate::app::App;
use anyhow::Result;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct Persistence;

impl Persistence {
    fn get_data_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "kronos", "kronos")
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;
        
        Ok(data_dir.join("state.json"))
    }

    pub fn save(app: &App) -> Result<()> {
        let path = Self::get_data_path()?;
        let json = serde_json::to_string_pretty(app)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load() -> Result<Option<App>> {
        let path = Self::get_data_path()?;
        
        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(path)?;
        let app: App = serde_json::from_str(&json)?;
        Ok(Some(app))
    }
}
