//! Application configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils::error::{Error, Result};

/// Application-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Selected theme
    pub theme: ThemeChoice,
    /// Clipboard timeout in seconds
    pub clipboard_timeout_secs: u64,
    /// Auto-lock enabled
    pub auto_lock_enabled: bool,
    /// Auto-lock timeout in seconds
    pub auto_lock_timeout_secs: u64,
    /// Show Nerd Font icons
    pub show_icons: bool,
    /// Enable mouse support
    pub mouse_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::CatppuccinMocha,
            clipboard_timeout_secs: 30,
            auto_lock_enabled: true,
            auto_lock_timeout_secs: 300, // 5 minutes
            show_icons: true,
            mouse_enabled: true,
        }
    }
}

impl AppConfig {
    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let contents = fs::read_to_string(&path)
                .map_err(|e| Error::FileRead(path.clone(), e))?;
            let config: AppConfig = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::FileWrite(path.clone(), e))?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)
            .map_err(|e| Error::FileWrite(path, e))
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "vault", "vault")
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))?;
        Ok(dirs.config_dir().join("config.json"))
    }

    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "vault", "vault")
            .ok_or_else(|| Error::Config("Could not determine data directory".to_string()))?;
        Ok(dirs.data_dir().to_path_buf())
    }

    /// Load configuration, falling back to defaults on error
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }
}

/// Available theme choices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    // Catppuccin variants
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    #[default]
    CatppuccinMocha,
    // TokyoNight variants
    TokyoNightNight,
    TokyoNightStorm,
    TokyoNightDay,
}

impl ThemeChoice {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeChoice::CatppuccinLatte => "Catppuccin Latte",
            ThemeChoice::CatppuccinFrappe => "Catppuccin Frappé",
            ThemeChoice::CatppuccinMacchiato => "Catppuccin Macchiato",
            ThemeChoice::CatppuccinMocha => "Catppuccin Mocha",
            ThemeChoice::TokyoNightNight => "Tokyo Night",
            ThemeChoice::TokyoNightStorm => "Tokyo Night Storm",
            ThemeChoice::TokyoNightDay => "Tokyo Night Day",
        }
    }

    /// Get all available themes
    pub fn all() -> &'static [ThemeChoice] {
        &[
            ThemeChoice::CatppuccinLatte,
            ThemeChoice::CatppuccinFrappe,
            ThemeChoice::CatppuccinMacchiato,
            ThemeChoice::CatppuccinMocha,
            ThemeChoice::TokyoNightNight,
            ThemeChoice::TokyoNightStorm,
            ThemeChoice::TokyoNightDay,
        ]
    }

    /// Check if this is a light theme
    pub fn is_light(&self) -> bool {
        matches!(
            self,
            ThemeChoice::CatppuccinLatte | ThemeChoice::TokyoNightDay
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.theme, ThemeChoice::CatppuccinMocha);
        assert_eq!(config.clipboard_timeout_secs, 30);
        assert!(config.auto_lock_enabled);
    }

    #[test]
    fn test_theme_display_name() {
        assert_eq!(
            ThemeChoice::CatppuccinMocha.display_name(),
            "Catppuccin Mocha"
        );
        assert_eq!(
            ThemeChoice::TokyoNightNight.display_name(),
            "Tokyo Night"
        );
    }

    #[test]
    fn test_theme_is_light() {
        assert!(ThemeChoice::CatppuccinLatte.is_light());
        assert!(ThemeChoice::TokyoNightDay.is_light());
        assert!(!ThemeChoice::CatppuccinMocha.is_light());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.theme, loaded.theme);
        assert_eq!(config.clipboard_timeout_secs, loaded.clipboard_timeout_secs);
    }
}
