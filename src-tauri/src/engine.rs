use crate::config::Config;
use crate::switch::{self, BtExecutor, Decision, Mode};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackpadStatus {
    Connected,
    Free,
    NotFound,
    HandingOff,
    OtherMac,
}

impl Default for TrackpadStatus {
    fn default() -> Self {
        TrackpadStatus::NotFound
    }
}

#[derive(Debug)]
pub struct AppState {
    pub mode: Mode,
    pub status: TrackpadStatus,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            mode: Mode::Auto,
            status: TrackpadStatus::NotFound,
        }
    }
}

#[derive(Clone)]
pub struct Engine {
    pub bt: Arc<dyn BtExecutor>,
    pub cfg: Arc<Mutex<Config>>,
    pub state: Arc<Mutex<AppState>>,
}

impl Engine {
    pub fn new(bt: Arc<dyn BtExecutor>, cfg: Config) -> Self {
        Engine {
            bt,
            cfg: Arc::new(Mutex::new(cfg)),
            state: Arc::new(Mutex::new(AppState::default())),
        }
    }
}

pub fn start(engine: Engine) {
    std::thread::spawn(move || loop {
        tick(&engine);
        std::thread::sleep(Duration::from_secs(3));
    });
}

pub fn tick(e: &Engine) {
    let addr = e.cfg.lock().unwrap().trackpad_address.clone();
    if addr.is_empty() {
        return;
    }

    let connected = e.bt.is_connected(&addr).unwrap_or(false);
    let mode = e.state.lock().unwrap().mode;

    if connected {
        let mut s = e.state.lock().unwrap();
        s.mode = Mode::Auto;
        s.status = TrackpadStatus::Connected;
        return;
    }

    let present = e
        .bt
        .inquiry(3)
        .unwrap_or_default()
        .iter()
        .any(|d| d.address == addr);

    let (decision, next_mode) = switch::decide(mode, connected, present);
    {
        let mut s = e.state.lock().unwrap();
        s.mode = next_mode;
    }

    match decision {
        Decision::Claim => {
            if let Err(err) = switch::claim(e.bt.as_ref(), &addr) {
                eprintln!("trackswitch: claim failed: {err}");
            } else {
                let mut s = e.state.lock().unwrap();
                s.status = TrackpadStatus::Connected;
            }
        }
        Decision::Nothing => {
            let status = if present {
                if mode == Mode::Yielded {
                    TrackpadStatus::HandingOff
                } else {
                    TrackpadStatus::Free
                }
            } else {
                TrackpadStatus::OtherMac
            };
            let mut s = e.state.lock().unwrap();
            s.status = status;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueutil::Device;
    use crate::config::Config;
    use crate::switch::{BtExecutor, Mode, SwitchError};
    use std::sync::Arc;

    struct FakeBt {
        connected: bool,
        present: bool,
    }

    impl BtExecutor for FakeBt {
        fn is_connected(&self, _a: &str) -> Result<bool, SwitchError> {
            Ok(self.connected)
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
            if self.present {
                Ok(vec![Device {
                    name: "Magic Trackpad".into(),
                    address: "aa:bb:cc:dd:ee:ff".into(),
                    paired: true,
                    connected: false,
                }])
            } else {
                Ok(vec![])
            }
        }
    }

    fn engine(bt: FakeBt, mode: Mode) -> Engine {
        let cfg = Config {
            trackpad_address: "aa:bb:cc:dd:ee:ff".into(),
            peer_name: "Mac B".into(),
            autostart: false,
        };
        let e = Engine::new(Arc::new(bt), cfg);
        e.state.lock().unwrap().mode = mode;
        e
    }

    #[test]
    fn tick_claims_a_free_trackpad() {
        let bt = FakeBt {
            connected: false,
            present: true,
        };
        let e = engine(bt, Mode::Auto);
        tick(&e);
        let s = e.state.lock().unwrap();
        assert_eq!(s.status, TrackpadStatus::Connected);
        assert_eq!(s.mode, Mode::Auto);
    }

    #[test]
    fn tick_does_not_claim_when_yielded() {
        let bt = FakeBt {
            connected: false,
            present: true,
        };
        let e = engine(bt, Mode::Yielded);
        tick(&e);
        let s = e.state.lock().unwrap();
        assert_eq!(s.status, TrackpadStatus::HandingOff);
        assert_eq!(s.mode, Mode::Yielded);
    }

    #[test]
    fn tick_rearms_when_yielded_and_trackpad_taken() {
        let bt = FakeBt {
            connected: false,
            present: false,
        };
        let e = engine(bt, Mode::Yielded);
        tick(&e);
        let s = e.state.lock().unwrap();
        assert_eq!(s.mode, Mode::Auto);
        assert_eq!(s.status, TrackpadStatus::OtherMac);
    }

    #[test]
    fn tick_sets_connected_when_trackpad_is_here() {
        let bt = FakeBt {
            connected: true,
            present: false,
        };
        let e = engine(bt, Mode::Yielded);
        tick(&e);
        let s = e.state.lock().unwrap();
        assert_eq!(s.status, TrackpadStatus::Connected);
        assert_eq!(s.mode, Mode::Auto);
    }
}
