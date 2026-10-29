pub mod blueutil;
pub mod cli;
pub mod commands;
pub mod config;
pub mod discovery;
pub mod engine;
pub mod peer;
pub mod setup;
pub mod switch;
pub mod update;

use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = config::load(&config::default_path()).unwrap_or_default();
    let needs_setup = cfg.trackpad_address.is_empty() || cfg.peer_name.is_empty();
    let engine = engine::Engine::new(Arc::new(switch::Executor), cfg);

    engine::start(engine.clone());
    let _ = peer::advertise();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::switch_to_peer,
            commands::take_trackpad,
            commands::list_devices,
            commands::discover_peer,
            commands::set_config
        ])
        .setup(move |app| {
            let peer = app
                .state::<engine::Engine>()
                .cfg
                .lock()
                .unwrap()
                .peer_name
                .clone();
            let peer_label = if peer.is_empty() {
                "Other Mac".to_string()
            } else {
                peer
            };

            let take_label = "This Mac (take trackpad)".to_string();
            let switch_label = format!("Switch to {peer_label}");

            let take_i = MenuItem::with_id(app, "take", take_label.as_str(), true, None::<&str>)?;
            let switch_i =
                MenuItem::with_id(app, "switch", switch_label.as_str(), true, None::<&str>)?;
            let prefs_i = MenuItem::with_id(app, "prefs", "Preferences", true, None::<&str>)?;
            let update_i =
                MenuItem::with_id(app, "update", "Check for updates", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&take_i, &switch_i, &prefs_i, &update_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let engine = app.state::<engine::Engine>();
                    match event.id.as_ref() {
                        "take" => {
                            let _ = commands::take_trackpad_impl(engine.inner());
                        }
                        "switch" => {
                            let _ = commands::switch_to_peer_impl(engine.inner());
                        }
                        "prefs" => open_preferences(app),
                        "update" => check_for_updates(app),
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app)?;
            if needs_setup {
                open_preferences(app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn check_for_updates(app: &tauri::AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match update::check_and_install(&app).await {
            Ok(true) => app.restart(),
            Ok(false) => {}
            Err(e) => eprintln!("update: {e}"),
        }
    });
}

fn open_preferences(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("prefs") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "prefs", WebviewUrl::App("index.html".into()))
        .title("TrackSwitch")
        .inner_size(440.0, 380.0)
        .build();
}
