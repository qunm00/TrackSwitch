use std::path::PathBuf;

pub fn plist_content() -> &'static str {
    include_str!("../../packaging/com.qunm00.trackswitch.plist")
}

pub fn launch_agents_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library").join("LaunchAgents")
}

pub fn install_autostart() -> Result<(), String> {
    let dir = launch_agents_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("com.qunm00.trackswitch.plist");
    std::fs::write(path, plist_content()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_has_required_keys() {
        let c = plist_content();
        assert!(c.contains("<key>RunAtLoad</key>"));
        assert!(c.contains("<key>KeepAlive</key>"));
        assert!(c.contains("<true/>"));
    }
}
