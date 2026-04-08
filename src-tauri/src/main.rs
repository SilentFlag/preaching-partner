// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod datatypes;

use tokio::sync::{mpsc, oneshot, broadcast};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use crate::datatypes::datatypes::{WsState, WsRequest, WsEvent, ClientMessage};
use tauri::State;

async fn connect_to_server(
    mut request_rx: mpsc::Receiver<WsRequest>,
    event_tx: broadcast::Sender<WsEvent>
) {
    let url = String::from("ws://127.0.0.1:9001");

    let (ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect");

    println!("Connected to server");

    let (mut write, mut read) = ws_stream.split();

    // TODO: RUSTLS ENCRYPTION

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let message = ClientMessage::Login {
        name: String::from("Ethan"),
        password: String::from("unset")
    };
    let message_bytes = rmp_serde::to_vec(&message).unwrap();
    write.send(tokio_tungstenite::tungstenite::Message::binary(message_bytes)).await;
    println!("Sent message");


    loop {
        tokio::select! {
            Some(req) = request_rx.recv() => {
                let _ = write.send(req.payload.clone().into()).await;

                // Wait for response (simple example)
                if let Some(msg) = read.next().await {
                    let msg = msg.unwrap().to_string();
                    let _ = req.response_tx.send(msg);
                }
            }

            // 🔹 Handle incoming unsolicited messages
            Some(msg) = read.next() => {
                let msg = msg.unwrap().to_string();
                let _ = event_tx.send(WsEvent { payload: msg });
            }
        }
    }

    // write.send(tokio_tungstenite::tungstenite::Message::Close(None));
}

// async fn listen_events(state: State<'_, WsState>) -> Result<(), String> {
//     let mut rx = state.event_tx.subscribe();

//     tokio::spawn(async move {
//         while let Ok(event) = rx.recv().await {
//             println!("Received event: {:?}", event);
//             // You can emit to frontend here using AppHandle
//         }
//     });

//     Ok(())
// }

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

fn main() {

    let (request_tx, request_rx) = mpsc::channel(32);
    let (event_tx, _) = broadcast::channel(32);

    let ws_state = WsState {
        request_tx,
        event_tx: event_tx.clone(),
    };

    tauri::async_runtime::spawn(async {
        connect_to_server(request_rx, event_tx).await;
    });

    run(ws_state);
}
