use crate::blueutil;
use crate::config;
use crate::switch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subcommand {
    Status,
    Switch,
    Setup,
}

pub fn parse_args(args: &[String]) -> Option<Subcommand> {
    match args.get(1).map(|s| s.as_str()) {
        Some("status") => Some(Subcommand::Status),
        Some("switch") => Some(Subcommand::Switch),
        Some("setup") => Some(Subcommand::Setup),
        _ => None,
    }
}

pub fn run_status() {
    let cfg = config::load(&config::default_path()).unwrap_or_default();
    if cfg.trackpad_address.is_empty() {
        println!("not configured");
        return;
    }
    match blueutil::is_connected(&cfg.trackpad_address) {
        Ok(true) => println!("connected to this Mac"),
        Ok(false) => println!("free or on the other Mac"),
        Err(e) => println!("error: {e}"),
    }
}

pub fn run_switch() -> Result<(), String> {
    let cfg = config::load(&config::default_path()).unwrap_or_default();
    if cfg.trackpad_address.is_empty() {
        return Err("no trackpad configured".into());
    }
    let bt = switch::Executor;
    switch::release(&bt, &cfg.trackpad_address).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subcommand() {
        assert_eq!(
            parse_args(&["trackswitch".into(), "status".into()]),
            Some(Subcommand::Status)
        );
        assert_eq!(
            parse_args(&["trackswitch".into(), "switch".into()]),
            Some(Subcommand::Switch)
        );
        assert_eq!(
            parse_args(&["trackswitch".into(), "setup".into()]),
            Some(Subcommand::Setup)
        );
    }

    #[test]
    fn no_subcommand_is_none() {
        assert_eq!(parse_args(&["trackswitch".into()]), None);
    }
}
