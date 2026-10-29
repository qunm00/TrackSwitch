use std::path::{Path, PathBuf};

pub const LABEL: &str = "com.qunm00.trackswitch";

pub fn launch_agents_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library").join("LaunchAgents")
}

pub fn plist_path() -> PathBuf {
    launch_agents_dir().join(format!("{LABEL}.plist"))
}

/// Build the LaunchAgent for this app.
///
/// The executable path comes from the app that writes the file, so the login
/// item follows the bundle when the app moves, for example from /Applications
/// to ~/Applications.
///
/// `KeepAlive` uses `SuccessfulExit: false`, so launchd restarts the app after a
/// crash but not after a clean quit. A plain `true` would start the app again
/// every time the user chose Quit, and it would fight the restart that follows
/// an update.
pub fn plist_content(executable: &Path) -> String {
    let exe = escape_xml(&executable.to_string_lossy());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
</dict>
</plist>
"#,
        label = LABEL,
        exe = exe,
    )
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn install_autostart() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(launch_agents_dir()).map_err(|e| e.to_string())?;
    std::fs::write(plist_path(), plist_content(&exe)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        plist_content(Path::new(
            "/Applications/TrackSwitch.app/Contents/MacOS/trackswitch",
        ))
    }

    #[test]
    fn plist_has_required_keys() {
        let c = sample();
        assert!(c.contains("<string>com.qunm00.trackswitch</string>"));
        assert!(c.contains("<key>RunAtLoad</key>"));
        assert!(c.contains("<key>KeepAlive</key>"));
        assert!(c.contains("<key>SuccessfulExit</key>"));
        assert!(c.contains("<false/>"));
    }

    #[test]
    fn plist_points_at_the_running_app() {
        let c = sample();
        assert!(c.contains("/Applications/TrackSwitch.app/Contents/MacOS/trackswitch"));
    }

    #[test]
    fn plist_follows_a_moved_app() {
        let c = plist_content(Path::new(
            "/Users/me/Applications/TrackSwitch.app/Contents/MacOS/trackswitch",
        ));
        assert!(c.contains(
            "<string>/Users/me/Applications/TrackSwitch.app/Contents/MacOS/trackswitch</string>"
        ));
        assert!(!c.contains("<string>/Applications/TrackSwitch"));
    }

    #[test]
    fn plist_escapes_xml() {
        let c = plist_content(Path::new("/tmp/a&b/<TrackSwitch>"));
        assert!(c.contains("/tmp/a&amp;b/&lt;TrackSwitch&gt;"));
    }
}
