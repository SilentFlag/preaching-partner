mod core;
mod database;
mod datatypes;
mod networking;
mod services;
mod sync;
// mod user_input;

use datatypes::{ClientPayload, FrontendReponse, ServerPayload, WsRequest, WsState};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{broadcast, mpsc, oneshot};

#[tauri::command]
async fn login(
    app_handle: AppHandle,
    state: State<'_, WsState>,
    username: String,
    password: String,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    let msg = ClientPayload::Login {
        name: username,
        password: password,
    };
    println!("{:?}", &msg);
    let request = WsRequest {
        payload: msg,
        response_tx: tx,
    };

    let sent_mess = state
        .request_tx
        .send(request)
        .await
        .map_err(|e| e.to_string());
    println!("message sent something {:?}", sent_mess);

    let response = rx.await.map_err(|e| e.to_string());
    println!("Recieved message to login function: {:?}", response);
    // TODO: handle error case
    let success = if let Ok(msg) = response {
        match msg.payload {
            ServerPayload::ConfirmLogin { success, .. } => success,
            _ => {
                // TODO: Handle this case
                println!("Unexpected message from server");
                false
            }
        }
    } else {
        // TODO: handle recieving message fail
        false
    };

    // TODO: Send message to frontend (webview)
    let payload = FrontendReponse::ConfirmLogin {
        success,
        name: "Default".to_string(), // or Some(token_string)
    };

    // TODO: Handle the error rather than crash with ?
    app_handle
        .emit("login-result", payload)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (request_tx, request_rx) = mpsc::channel(32);
    let (event_tx, _) = broadcast::channel(32);

    let ws_state = WsState {
        request_tx,
        event_tx: event_tx.clone(),
    };

    tauri::async_runtime::spawn(async {
        core::initiate_backend(request_rx, event_tx).await;
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
