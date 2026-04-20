pub mod datatypes;
pub mod networking;

use tokio::sync::{mpsc, oneshot, broadcast};
use datatypes::{WsState, WsRequest, ClientPayload, ServerPayload};
use tauri::State;

#[tauri::command]
async fn login(state: State<'_, WsState>, username: String, password: String) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    let msg = ClientPayload::Login {name: username, password: password};
    println!("{:?}", &msg);
    let request = WsRequest {
        payload: msg,
        response_tx: tx,
    };

    let sent_mess = state.request_tx.send(request).await.map_err(|e| e.to_string());
    println!("message sent something {:?}", sent_mess);

    let response = rx.await.map_err(|e| e.to_string());
    println!("Recieved message to login function: {:?}", response);
    // TODO: handle error case
    let (refresh_token, access_token) = if let Ok(msg) = response {
        match msg.payload {
            ServerPayload::ConfirmLogin{success, refresh_token, access_token} => {
                println!("Server response had a payload with a confirm value of {:?}, {:?}, {:?}", success, refresh_token, access_token);
                (refresh_token, access_token)
            }
            _ => {
                println!("Unexpected message from server");
                (None, None)
            }
        }
    }
    // TODO: store tokens
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

