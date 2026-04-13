use std::path::PathBuf;

/// Returns the platform-appropriate steno config directory.
/// - Windows: %APPDATA%\steno
/// - Linux:   ~/.config/steno
/// - macOS:   ~/Library/Application Support/steno
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steno")
}

/// Returns the directory where user-installed dictionary packs live.
pub fn dicts_dir() -> PathBuf {
    config_dir().join("dicts")
}

/// Returns the path to the user's personal extension dictionary.
pub fn personal_dict_path() -> PathBuf {
    config_dir().join("personal.toml")
}

/// Returns the path to the cumulative gain stats file.
pub fn gain_path() -> PathBuf {
    config_dir().join("gain.toml")
}

/// Returns the path to the phrase usage frequency file (used by steno learn/suggest).
pub fn usage_path() -> PathBuf {
    config_dir().join("usage.toml")
}

/// Ensure config directories exist (creates them if missing).
pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(dicts_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_is_absolute() {
        let dir = config_dir();
        assert!(dir.is_absolute(), "config dir must be absolute: {:?}", dir);
    }

    #[test]
    fn test_dicts_dir_is_under_config() {
        let dicts = dicts_dir();
        let config = config_dir();
        assert!(dicts.starts_with(&config), "dicts dir must be under config dir");
    }

    #[test]
    fn test_personal_dict_is_toml() {
        let p = personal_dict_path();
        assert_eq!(p.extension().and_then(|e| e.to_str()), Some("toml"));
    }
}
