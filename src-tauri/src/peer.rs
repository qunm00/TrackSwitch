use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const SERVICE_TYPE: &str = "_trackswitch._tcp.local.";

pub fn normalize_peer_name(name: &str) -> String {
    name.trim()
        .trim_end_matches('.')
        .trim_end_matches(".local")
        .trim_end_matches('.')
        .to_string()
}

fn hostname() -> String {
    if let Ok(h) = std::env::var("HOSTNAME") {
        if !h.trim().is_empty() {
            return h;
        }
    }
    if let Ok(out) = std::process::Command::new("hostname").output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return s;
        }
    }
    "unknown".to_string()
}

pub fn advertise() -> Result<(), String> {
    let mdns = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let host = hostname();
    let mut props = HashMap::new();
    props.insert("peer".to_string(), host.clone());
    let info = ServiceInfo::new(SERVICE_TYPE, "trackswitch", &host, "127.0.0.1", 0, props)
        .map_err(|e| e.to_string())?;
    mdns.register(info).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn discover_peer(timeout: Duration) -> Option<String> {
    let mdns = ServiceDaemon::new().ok()?;
    let receiver = mdns.browse(SERVICE_TYPE).ok()?;
    let start = Instant::now();
    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        let wait = remaining.min(Duration::from_millis(500));
        if let Ok(ServiceEvent::ServiceResolved(info)) = receiver.recv_timeout(wait) {
            if let Some(name) = info.get_property_val_str("peer") {
                let name = name.to_string();
                if name != hostname() {
                    return Some(normalize_peer_name(&name));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_dot_from_mdns_name() {
        assert_eq!(normalize_peer_name("MacBook-Pro.local."), "MacBook-Pro");
    }

    #[test]
    fn strips_local_suffix() {
        assert_eq!(normalize_peer_name("MacBook-Pro.local"), "MacBook-Pro");
    }
}
