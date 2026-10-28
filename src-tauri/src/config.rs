use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub trackpad_address: String,
    pub peer_name: String,
    pub autostart: bool,
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn save(path: &Path, cfg: &Config) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    let data = serde_json::to_string_pretty(cfg)?;
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn default_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("trackswitch")
        .join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let cfg = Config {
            trackpad_address: "aa:bb:cc:dd:ee:ff".into(),
            peer_name: "Mac B".into(),
            autostart: false,
        };
        let p = std::env::temp_dir().join("trackswitch-test-config.json");
        save(&p, &cfg).unwrap();
        let got = load(&p).unwrap();
        assert_eq!(got.trackpad_address, cfg.trackpad_address);
        assert_eq!(got.peer_name, cfg.peer_name);
        assert!(!got.autostart);
    }

    #[test]
    fn loads_default_when_missing() {
        let p = std::env::temp_dir().join("trackswitch-test-config-missing.json");
        let _ = std::fs::remove_file(&p);
        let got = load(&p).unwrap();
        assert_eq!(got.trackpad_address, "");
        assert_eq!(got.peer_name, "");
    }
}
