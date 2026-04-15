//! Application configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    /// Custom icon color
    pub icon_color: IconColorChoice,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::CatppuccinMocha,
            clipboard_timeout_secs: 60,
            auto_lock_enabled: true,
            auto_lock_timeout_secs: 300, // 5 minutes
            show_icons: true,
            mouse_enabled: true,
            icon_color: IconColorChoice::ThemeDefault,
        }
    }
}

impl AppConfig {
    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let contents =
                fs::read_to_string(&path).map_err(|e| Error::FileRead(path.clone(), e))?;
            let config: AppConfig = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        write_atomic_secure(&path, contents.as_bytes())
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

fn set_secure_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    }

    Ok(())
}

fn create_secure_file(path: &Path) -> Result<fs::File> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    }

    #[cfg(not(unix))]
    {
        fs::File::create(path).map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    }
}

fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            fs::File::open(parent)
                .and_then(|dir| dir.sync_all())
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        }
    }
    Ok(())
}

fn write_atomic_secure(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp_name = format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config"),
        uuid::Uuid::new_v4()
    );
    let tmp_path = parent.join(tmp_name);

    let mut file = create_secure_file(&tmp_path)?;
    let write_result = (|| {
        file.write_all(contents)
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        file.sync_all()
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    })();

    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    drop(file);
    fs::rename(&tmp_path, path).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    set_secure_permissions(path)?;
    sync_parent_dir(path)?;
    Ok(())
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
        assert_eq!(config.clipboard_timeout_secs, 60);
        assert!(config.auto_lock_enabled);
    }

    #[test]
    fn test_theme_display_name() {
        assert_eq!(
            ThemeChoice::CatppuccinMocha.display_name(),
            "Catppuccin Mocha"
        );
        assert_eq!(ThemeChoice::TokyoNightNight.display_name(), "Tokyo Night");
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

    #[cfg(unix)]
    #[test]
    fn test_set_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{}").unwrap();

        set_secure_permissions(&path).unwrap();
        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_write_atomic_secure_writes_content() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        write_atomic_secure(&path, br#"{"theme":"catppuccin_mocha"}"#).unwrap();

        let contents = std::fs::read_to_string(path).unwrap();
        assert_eq!(contents, r#"{"theme":"catppuccin_mocha"}"#);
    }

    #[cfg(unix)]
    #[test]
    fn test_write_atomic_secure_sets_permissions() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        write_atomic_secure(&path, b"{}").unwrap();

        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}

/// Available icon color choices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IconColorChoice {
    #[default]
    ThemeDefault,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
}

impl IconColorChoice {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            IconColorChoice::ThemeDefault => "Theme Default",
            IconColorChoice::Red => "Red",
            IconColorChoice::Green => "Green",
            IconColorChoice::Blue => "Blue",
            IconColorChoice::Yellow => "Yellow",
            IconColorChoice::Magenta => "Magenta",
            IconColorChoice::Cyan => "Cyan",
            IconColorChoice::White => "White",
        }
    }

    /// Get all available icon colors
    pub fn all() -> &'static [IconColorChoice] {
        &[
            IconColorChoice::ThemeDefault,
            IconColorChoice::Red,
            IconColorChoice::Green,
            IconColorChoice::Blue,
            IconColorChoice::Yellow,
            IconColorChoice::Magenta,
            IconColorChoice::Cyan,
            IconColorChoice::White,
        ]
    }

    /// Convert to ratatui Color
    pub fn to_color(&self, theme: &crate::ui::theme::ThemePalette) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            IconColorChoice::ThemeDefault => theme.accent,
            IconColorChoice::Red => Color::Red,
            IconColorChoice::Green => Color::Green,
            IconColorChoice::Blue => Color::Blue,
            IconColorChoice::Yellow => Color::Yellow,
            IconColorChoice::Magenta => Color::Magenta,
            IconColorChoice::Cyan => Color::Cyan,
            IconColorChoice::White => Color::White,
        }
    }
}
