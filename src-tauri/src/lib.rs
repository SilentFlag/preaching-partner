pub mod datatypes;
pub mod networking;

use tokio::sync::{mpsc, oneshot, broadcast};
use datatypes::{WsState, WsRequest};
use tauri::State;

#[tauri::command]
async fn login(state: State<'_, WsState>, username: String, password: String) -> Result<(), bool> {
    let (tx, rx) = oneshot::channel();
    let msg = String::from(format!("Login by {:?} pass {:?}", username, password));
    println!("{}", &msg);
    let request = WsRequest {
        payload: msg,
        response_tx: tx,
    };

    let _ = state.request_tx.send(request).await.map_err(|e| e.to_string());

    let _response = rx.await.map_err(|e| e.to_string());
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(ws_state: WsState) {

    let (request_tx, request_rx) = mpsc::channel(32);
    let (event_tx, _) = broadcast::channel(32);

    let ws_state = WsState {
        request_tx,
        event_tx: event_tx.clone(),
    };

    tauri::async_runtime::spawn(async {
        networking::connect_to_server(request_rx, event_tx).await;
    });

    tauri::Builder::default()
        .manage(ws_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login])
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

