use crate::blueutil;
use crate::config;
use crate::engine::{Engine, TrackpadStatus};
use crate::peer;
use crate::setup;
use crate::switch::{self, Mode};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct StatusPayload {
    pub status: String,
    pub peer_name: String,
    pub trackpad_address: String,
}

#[derive(Debug, Serialize)]
pub struct BluetoothDevicePayload {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

fn status_name(status: TrackpadStatus) -> &'static str {
    match status {
        TrackpadStatus::Connected => "connected",
        TrackpadStatus::Free => "free",
        TrackpadStatus::NotFound => "not_found",
        TrackpadStatus::HandingOff => "handing_off",
        TrackpadStatus::OtherMac => "other_mac",
    }
}

pub fn status_payload(engine: &Engine) -> StatusPayload {
    let cfg = engine.cfg.lock().unwrap();
    let state = engine.state.lock().unwrap();
    StatusPayload {
        status: status_name(state.status).to_string(),
        peer_name: cfg.peer_name.clone(),
        trackpad_address: cfg.trackpad_address.clone(),
    }
}

pub fn switch_to_peer_impl(engine: &Engine) -> Result<(), String> {
    let addr = engine.cfg.lock().unwrap().trackpad_address.clone();
    if addr.is_empty() {
        return Err("no trackpad configured".into());
    }
    switch::release(engine.bt.as_ref(), &addr).map_err(|e| e.to_string())?;
    engine.state.lock().unwrap().mode = Mode::Yielded;
    Ok(())
}

pub fn take_trackpad_impl(engine: &Engine) -> Result<(), String> {
    let addr = engine.cfg.lock().unwrap().trackpad_address.clone();
    if addr.is_empty() {
        return Err("no trackpad configured".into());
    }
    switch::claim(engine.bt.as_ref(), &addr).map_err(|e| e.to_string())?;
    let mut s = engine.state.lock().unwrap();
    s.mode = Mode::Auto;
    s.status = TrackpadStatus::Connected;
    Ok(())
}

pub fn list_devices_impl() -> Result<Vec<BluetoothDevicePayload>, String> {
    blueutil::paired()
        .map(|devices| {
            devices
                .into_iter()
                .map(|d| BluetoothDevicePayload {
                    name: d.name,
                    address: d.address,
                    connected: d.connected,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn status(engine: State<'_, Engine>) -> Result<StatusPayload, String> {
    Ok(status_payload(&engine))
}

#[tauri::command]
pub fn switch_to_peer(engine: State<'_, Engine>) -> Result<(), String> {
    switch_to_peer_impl(&engine)
}

#[tauri::command]
pub fn take_trackpad(engine: State<'_, Engine>) -> Result<(), String> {
    take_trackpad_impl(&engine)
}

#[tauri::command]
pub fn list_devices() -> Result<Vec<BluetoothDevicePayload>, String> {
    list_devices_impl()
}

#[tauri::command]
pub fn discover_peer() -> Result<Option<String>, String> {
    Ok(peer::discover_peer(std::time::Duration::from_secs(3)))
}

#[tauri::command]
pub fn set_config(
    engine: State<'_, Engine>,
    trackpad_address: String,
    peer_name: String,
) -> Result<(), String> {
    let mut cfg = engine.cfg.lock().unwrap();
    cfg.trackpad_address = trackpad_address;
    cfg.peer_name = peer_name;
    config::save(&config::default_path(), &cfg).map_err(|e| e.to_string())?;
    let _ = setup::install_autostart();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueutil::Device;
    use crate::config::Config;
    use crate::engine::{Engine, TrackpadStatus};
    use crate::switch::{BtExecutor, Mode, SwitchError};
    use std::sync::Arc;

    struct FakeBt;

    impl BtExecutor for FakeBt {
        fn is_connected(&self, _a: &str) -> Result<bool, SwitchError> {
            Ok(false)
        }
        fn pair(&self, _a: &str) -> Result<(), SwitchError> {
            Ok(())
        }
        fn connect(&self, _a: &str) -> Result<(), SwitchError> {
            Ok(())
        }
        fn unpair(&self, _a: &str) -> Result<(), SwitchError> {
            Ok(())
        }
        fn disconnect(&self, _a: &str) -> Result<(), SwitchError> {
            Ok(())
        }
        fn set_power(&self, _on: bool) -> Result<(), SwitchError> {
            Ok(())
        }
        fn inquiry(&self, _s: u8) -> Result<Vec<Device>, SwitchError> {
            Ok(vec![])
        }
    }

    fn engine_with(peer: &str) -> Engine {
        Engine::new(
            Arc::new(FakeBt),
            Config {
                trackpad_address: "aa:bb:cc:dd:ee:ff".into(),
                peer_name: peer.into(),
                autostart: false,
            },
        )
    }

    #[test]
    fn status_reports_peer_name() {
        let e = engine_with("Mac B");
        let p = status_payload(&e);
        assert_eq!(p.peer_name, "Mac B");
    }

    #[test]
    fn switch_to_peer_sets_yielded() {
        let e = engine_with("Mac B");
        switch_to_peer_impl(&e).unwrap();
        assert_eq!(e.state.lock().unwrap().mode, Mode::Yielded);
    }

    #[test]
    fn take_trackpad_sets_connected() {
        let e = engine_with("Mac B");
        take_trackpad_impl(&e).unwrap();
        let s = e.state.lock().unwrap();
        assert_eq!(s.mode, Mode::Auto);
        assert_eq!(s.status, TrackpadStatus::Connected);
    }
}
