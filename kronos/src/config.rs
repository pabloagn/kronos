use anyhow::{Context, Result};
use directories::ProjectDirs;
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub theme: Theme,
    pub icons: Icons,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Theme {
    #[serde(deserialize_with = "hex_to_color")]
    pub background: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub foreground: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub selection: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub black: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub red: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub green: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub yellow: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub blue: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub magenta: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub cyan: Color,
    #[serde(deserialize_with = "hex_to_color")]
    pub gray: Color,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Icons {
    pub global_timer: String,
    pub task_list: String,
    pub play: String,
    pub pause: String,
    pub stop: String,
    pub pending: String,
    pub done: String,
    pub select: String,
    pub progress_filled: String,
    pub progress_empty: String,
    pub input_cursor: String,
    pub separator: String,
    pub header_left: String,
    pub header_right: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            icons: Icons::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(9, 14, 19),
            foreground: Color::Rgb(197, 201, 199),
            selection: Color::Rgb(230, 195, 132),
            black: Color::Rgb(13, 12, 12),
            red: Color::Rgb(228, 104, 118),
            green: Color::Rgb(138, 154, 123),
            yellow: Color::Rgb(196, 178, 138),
            blue: Color::Rgb(127, 180, 202),
            magenta: Color::Rgb(162, 146, 163),
            cyan: Color::Rgb(122, 168, 159),
            gray: Color::Rgb(164, 167, 164),
        }
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            global_timer: "Δ".to_string(),
            task_list: "⬢".to_string(),
            play: "▶".to_string(),
            pause: "⏸".to_string(),
            stop: "■".to_string(),
            pending: "☐".to_string(),
            done: "☑".to_string(),
            select: "▸".to_string(),
            progress_filled: "█".to_string(),
            progress_empty: "░".to_string(),
            input_cursor: "▊".to_string(),
            separator: "│".to_string(),
            header_left: "⟪ ".to_string(),
            header_right: " ⟫".to_string(),
        }
    }
}

fn hex_to_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    if !s.starts_with('#') || s.len() != 7 {
        return Err(serde::de::Error::custom("invalid hex color format"));
    }
    let r = u8::from_str_radix(&s[1..3], 16).map_err(serde::de::Error::custom)?;
    let g = u8::from_str_radix(&s[3..5], 16).map_err(serde::de::Error::custom)?;
    let b = u8::from_str_radix(&s[5..7], 16).map_err(serde::de::Error::custom)?;
    Ok(Color::Rgb(r, g, b))
}

pub fn load_config() -> Result<Config> {
    match ProjectDirs::from("com", "pabloagn", "Kronos") {
        Some(proj_dirs) => {
            let path = proj_dirs.config_dir().join("kronos.toml");
            if path.exists() {
                let config_str = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file at {:?}", path))?;
                toml::from_str(&config_str)
                    .with_context(|| format!("Failed to parse config file at {:?}", path))
            } else {
                Ok(Config::default())
            }
        }
        None => Ok(Config::default()),
    }
}
