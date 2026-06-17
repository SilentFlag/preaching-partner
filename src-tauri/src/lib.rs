mod core;
mod database;
mod datatypes;
mod networking;
mod services;
mod sync;
mod user_input;

use datatypes::WsState;
use std::sync::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::datatypes::StartupState;
use user_input::{app_loaded, login};

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
        .invoke_handler(tauri::generate_handler![login, app_loaded])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            match event {
                tauri::RunEvent::Exit => {
                    // Your final cleanup code goes here
                    println!("Tauri app is exiting. Performing final cleanup.");
                }
                // Match other events or use a wildcard arm for non-exhaustive enum
                _ => {}
            }
        });
}
