use crate::blueutil::{self, BlueutilError, Device};

#[derive(Debug, thiserror::Error)]
pub enum SwitchError {
    #[error(transparent)]
    Bt(#[from] BlueutilError),
}

pub trait BtExecutor: Send + Sync {
    fn is_connected(&self, addr: &str) -> Result<bool, SwitchError>;
    fn pair(&self, addr: &str) -> Result<(), SwitchError>;
    fn connect(&self, addr: &str) -> Result<(), SwitchError>;
    fn unpair(&self, addr: &str) -> Result<(), SwitchError>;
    fn disconnect(&self, addr: &str) -> Result<(), SwitchError>;
    fn set_power(&self, on: bool) -> Result<(), SwitchError>;
    fn inquiry(&self, seconds: u8) -> Result<Vec<Device>, SwitchError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Auto,
    Yielded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Nothing,
    Claim,
}

pub fn release(bt: &dyn BtExecutor, addr: &str) -> Result<(), SwitchError> {
    bt.unpair(addr)?;
    bt.set_power(false)?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    bt.set_power(true)?;
    Ok(())
}

pub fn claim(bt: &dyn BtExecutor, addr: &str) -> Result<(), SwitchError> {
    // Pairing can fail when the device is not ready; the poll loop retries.
    if bt.pair(addr).is_err() {
        return Ok(());
    }
    bt.connect(addr)
}

pub fn decide(mode: Mode, connected_to_me: bool, present: bool) -> (Decision, Mode) {
    if connected_to_me {
        return (Decision::Nothing, Mode::Auto);
    }
    match mode {
        Mode::Yielded => {
            if present {
                (Decision::Nothing, Mode::Yielded)
            } else {
                (Decision::Nothing, Mode::Auto)
            }
        }
        Mode::Auto => {
            if present {
                (Decision::Claim, Mode::Auto)
            } else {
                (Decision::Nothing, Mode::Auto)
            }
        }
    }
}

pub struct Executor;

impl BtExecutor for Executor {
    fn is_connected(&self, addr: &str) -> Result<bool, SwitchError> {
        Ok(blueutil::is_connected(addr)?)
    }
    fn pair(&self, addr: &str) -> Result<(), SwitchError> {
        Ok(blueutil::pair(addr)?)
    }
    fn connect(&self, addr: &str) -> Result<(), SwitchError> {
        Ok(blueutil::connect(addr)?)
    }
    fn unpair(&self, addr: &str) -> Result<(), SwitchError> {
        Ok(blueutil::unpair(addr)?)
    }
    fn disconnect(&self, addr: &str) -> Result<(), SwitchError> {
        Ok(blueutil::disconnect(addr)?)
    }
    fn set_power(&self, on: bool) -> Result<(), SwitchError> {
        Ok(blueutil::set_power(on)?)
    }
    fn inquiry(&self, seconds: u8) -> Result<Vec<Device>, SwitchError> {
        Ok(blueutil::inquiry(seconds)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueutil::{BlueutilError, Device};

    struct FakeBt {
        connected: bool,
        pair_fails: bool,
    }

    impl BtExecutor for FakeBt {
        fn is_connected(&self, _a: &str) -> Result<bool, SwitchError> {
            Ok(self.connected)
        }
        fn pair(&self, _a: &str) -> Result<(), SwitchError> {
            if self.pair_fails {
                Err(SwitchError::Bt(BlueutilError::Parse("pair failed".into())))
            } else {
                Ok(())
            }
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

    #[test]
    fn claims_when_free_and_present() {
        assert_eq!(decide(Mode::Auto, false, true), (Decision::Claim, Mode::Auto));
    }

    #[test]
    fn does_nothing_when_connected_to_me() {
        assert_eq!(decide(Mode::Auto, true, true), (Decision::Nothing, Mode::Auto));
        assert_eq!(decide(Mode::Yielded, true, true), (Decision::Nothing, Mode::Auto));
    }

    #[test]
    fn yielded_mac_does_not_reclaim_a_free_trackpad() {
        assert_eq!(
            decide(Mode::Yielded, false, true),
            (Decision::Nothing, Mode::Yielded)
        );
    }

    #[test]
    fn yielded_mac_rearms_once_the_trackpad_is_taken() {
        assert_eq!(decide(Mode::Yielded, false, false), (Decision::Nothing, Mode::Auto));
    }

    #[test]
    fn claim_treats_pair_failure_as_ok() {
        let bt = FakeBt {
            connected: false,
            pair_fails: true,
        };
        assert!(claim(&bt, "aa:bb:cc:dd:ee:ff").is_ok());
    }
}
