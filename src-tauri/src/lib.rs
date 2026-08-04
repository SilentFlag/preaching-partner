/// Startup functions for the app
mod core;
/// Database abstraction and functions
mod database;
/// Custom datatypes
mod datatypes;
/// Websocket networking for users that are logged in
mod networking;
///
mod services;
/// Functions related to syncing the user when they come online
mod sync;
/// Functions called by the front end without user input
mod tauri_commands;
/// Functions called by the front end as a result of user input
mod user_input;

use datatypes::WsState;
use std::sync::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::datatypes::StartupState;
use tauri_commands::{app_loaded, get_maps};
use user_input::{get_map_data, login, complete_address};

/// Initialise and run the app, this function is run by the main.rs entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (request_tx, request_rx) = mpsc::channel(32);
    let (event_tx, _event_rx) = broadcast::channel(32);

    let ws_state = WsState {
        request_tx,
        event_tx: event_tx.clone(),
    };

    let startup_state = StartupState {
        request_rx: Mutex::new(Some(request_rx)),
        event_tx,
    };

    tauri::Builder::default()
        .manage(ws_state)
        .manage(startup_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            login,
            app_loaded,
            get_maps,
            get_map_data,
            complete_address,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::Exit => {
                    // Your final cleanup code goes here
                    println!("Tauri app is exiting. Performing final cleanup.");
                    app_handle.exit(0);
                }
                // Match other events or use a wildcard arm for non-exhaustive enum
                _ => {}
            }
        });
}
