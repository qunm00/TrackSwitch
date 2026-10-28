#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match trackswitch_lib::cli::parse_args(&args) {
        Some(trackswitch_lib::cli::Subcommand::Status) => {
            trackswitch_lib::cli::run_status();
        }
        Some(trackswitch_lib::cli::Subcommand::Switch) => {
            if let Err(e) = trackswitch_lib::cli::run_switch() {
                eprintln!("{e}");
                std::process::exit(1);
            }
            println!("released");
        }
        Some(trackswitch_lib::cli::Subcommand::Setup) | None => {
            trackswitch_lib::run();
        }
    }
}
