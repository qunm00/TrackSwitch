use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum BlueutilError {
    #[error("blueutil not found in PATH")]
    NotFound,
    #[error("blueutil exited with {status}: {stderr}")]
    CommandFailed { status: String, stderr: String },
    #[error("failed to parse blueutil output: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub name: String,
    pub address: String,
    pub paired: bool,
    pub connected: bool,
}

pub type DeviceInfo = Device;

fn run(args: &[&str]) -> Result<String, BlueutilError> {
    let output = Command::new("blueutil").args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            BlueutilError::NotFound
        } else {
            BlueutilError::Io(e)
        }
    })?;
    if !output.status.success() {
        return Err(BlueutilError::CommandFailed {
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_device(value: &serde_json::Value) -> Result<Device, BlueutilError> {
    let name = value
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let address = value
        .get("address")
        .and_then(|a| a.as_str())
        .ok_or_else(|| BlueutilError::Parse("missing address field".into()))?
        .to_string();
    let paired = value.get("paired").and_then(|p| p.as_bool()).unwrap_or(false);
    let connected = value
        .get("connected")
        .and_then(|c| c.as_bool())
        .unwrap_or(false);
    Ok(Device {
        name,
        address,
        paired,
        connected,
    })
}

pub fn parse_devices(json: &str) -> Result<Vec<Device>, BlueutilError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| BlueutilError::Parse(e.to_string()))?;
    let arr = value
        .as_array()
        .ok_or_else(|| BlueutilError::Parse("expected a JSON array".into()))?;
    arr.iter().map(parse_device).collect()
}

pub fn parse_info(json: &str) -> Result<DeviceInfo, BlueutilError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| BlueutilError::Parse(e.to_string()))?;
    parse_device(&value)
}

pub fn paired() -> Result<Vec<Device>, BlueutilError> {
    parse_devices(&run(&["--paired", "--format", "json"])?)
}

pub fn inquiry(seconds: u8) -> Result<Vec<Device>, BlueutilError> {
    let s = seconds.to_string();
    parse_devices(&run(&["--inquiry", s.as_str(), "--format", "json"])?)
}

pub fn is_connected(addr: &str) -> Result<bool, BlueutilError> {
    Ok(run(&["--is-connected", addr])?.trim() == "1")
}

pub fn info(addr: &str) -> Result<DeviceInfo, BlueutilError> {
    parse_info(&run(&["--info", addr, "--format", "json"])?)
}

pub fn pair(addr: &str) -> Result<(), BlueutilError> {
    run(&["--pair", addr]).map(|_| ())
}

pub fn unpair(addr: &str) -> Result<(), BlueutilError> {
    run(&["--unpair", addr]).map(|_| ())
}

pub fn connect(addr: &str) -> Result<(), BlueutilError> {
    run(&["--connect", addr]).map(|_| ())
}

pub fn disconnect(addr: &str) -> Result<(), BlueutilError> {
    run(&["--disconnect", addr]).map(|_| ())
}

pub fn set_power(on: bool) -> Result<(), BlueutilError> {
    run(&["--power", if on { "1" } else { "0" }]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_paired_json() {
        let raw = include_str!("../tests/fixtures/paired.json");
        let devices = parse_devices(raw).unwrap();
        assert_eq!(devices.len(), 2);
        let t = devices.iter().find(|d| d.name.contains("Trackpad")).unwrap();
        assert_eq!(t.address, "aa-bb-cc-dd-ee-ff");
        assert!(t.paired);
        assert!(t.connected);
    }

    #[test]
    fn parses_info_json() {
        let raw = include_str!("../tests/fixtures/info.json");
        let info = parse_info(raw).unwrap();
        assert_eq!(info.address, "aa-bb-cc-dd-ee-ff");
        assert_eq!(info.name, "Magic Trackpad");
        assert!(info.connected);
    }

    #[test]
    fn treats_null_name_as_empty() {
        let raw = r#"[{"address":"11-22-33-44-55-66","name":null,"paired":true,"connected":false}]"#;
        let devices = parse_devices(raw).unwrap();
        assert_eq!(devices[0].name, "");
    }
}
