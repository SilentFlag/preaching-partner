use crate::database::MyDatabase;
use crate::datatypes::{
    ClientMessage, ClientPayload, ServerMessage, ServerPayload, WsEvent, WsRequest, WsSender,
};
use crate::services;
use crate::sync::sync_with_server;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

// pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub async fn connect_to_server(
    mut request_rx: mpsc::Receiver<WsRequest>,
    _event_tx: broadcast::Sender<WsEvent>,
    db: MyDatabase, // DB functions must not be called directly, instead they should be called through the services module, this is to ensure that all database interactions are properly authenticated, logged and handled
    _app_handle: AppHandle,
) {
    // Create Connection
    let url = String::from("ws://127.0.0.1:9001/ws");

    // TODO: Handle connection errors, retry connection with exponential backoff, alert user if connection fails after multiple attempts
    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

    println!("Connected to server");

    let (write, read) = ws_stream.split();

    let mut sender: WsSender = WsSender { write, read };

    // TODO: RUSTLS ENCRYPTION

    let refresh_token = services::is_logged_in(db.clone()).await;
    if let Some(refresh_token) = refresh_token {
        let mut access_token = services::get_access_token(db.clone()).await;

        // Request to sync after initial setup of db and connection has been established
        // TODO: Set the time to the last time it has synced rather than a concrete value
        // TODO: use access token instead of user_id
        let mut current_id: u32 = 1; // Message id, don't start at 0, 0 indicates global message
        let mut response_senders = HashMap::new();

        let msg: ClientMessage = ClientMessage {
            // form message to send
            id: 0,
            access_token: access_token,
            payload: ClientPayload::RequestSync(0),
        };

        let send_result = sender.send(msg, refresh_token, db.clone()).await;
        match send_result {
            Ok(new_token) => access_token = new_token,
            Err(_) => { // TODO: handle error
            }
        }
        println!("sent sync request");

        // Core loop

        loop {
            tokio::select! {
                // handle io messages
                Some(req) = request_rx.recv() => {

                    let client_payload: ClientPayload = req.payload.into(); // extract payload

                    // Check for perms to do action
                    match client_payload {
                        ClientPayload::Login{name: _, password: _} => {
                            // TODO: Handle case, this should not be reached
                            continue;
                        }
                        _ => {}
                    }

                    response_senders.insert(current_id, req.response_tx); // remember reponse_tx for later in hashmap

                    let msg: ClientMessage = ClientMessage { // form message to send
                        id: current_id,
                        access_token: access_token,
                        payload: client_payload,
                    };
                    let send_result: Result<Option<[u8; 32]>, _> = sender.send(msg, refresh_token, db.clone()).await;
                    match send_result {
                        Ok(new_token) => access_token = new_token,
                        Err(_) => { // TODO: handle error
                        }
                    }
                    current_id += 1;

                    println!("Sent server message")
                }

                // Handle incoming messages
                Some(msg) = sender.read.next() => {
                    let coded_msg = msg;
                    match coded_msg {
                        Ok(coded_msg) => {

                            if let Message::Binary(bin) = coded_msg {
                                let msg: Result<ServerMessage, rmp_serde::decode::Error> = rmp_serde::from_slice(&bin);

                                match msg {
                                    Ok(msg) => {

                                        let response_msg = msg.clone();

                                        let _timestamp: u32 = msg.timestamp;

                                        // Check for messages that require db writes
                                        // TODO: unsure why the message being unknown_error crashes it
                                        match msg.payload {
                                            ServerPayload::ConfirmLogin{success: _, refresh_token, access_token} => {

                                                // TODO: Handle Errors
                                                if let Some(refresh_token_ok) = refresh_token {
                                                    let _refresh_result = services::save_refresh_token(db.clone(), refresh_token_ok).await;
                                                }

                                                if let Some(access_token_ok) = access_token {
                                                    let _access_token_result = services::save_access_token(db.clone(), access_token_ok).await;
                                                }
                                            }
                                            ServerPayload::SyncInformation(sync_info) => {
                                                let sync_result = sync_with_server(db.clone(), sync_info).await;
                                                if let Err(_error) = sync_result {
                                                    // TODO: handle error
                                                }
                                            }
                                            _ => {
                                                // TODO: log unexpected message
                                                println!("unexpected message");
                                                continue;
                                            }
                                            // Send response back to original caller
                                        }

                                        // Send message to the IO function which sent a request to the server
                                        if msg.id == 0 {
                                            // TODO: Broadcast a message to event listeners

                                            // let send_results = event_tx.send(WsEvent { payload: response_msg });
                                            // println!("failed to send message: {:?}", send_results);
                                        } else {
                                            let response_tx = response_senders.remove(&msg.id);
                                            match response_tx {
                                                Some(response_tx) => {
                                                    let _ = response_tx.send(response_msg);
                                                }
                                                _ => {
                                                    // TODO: handle error
                                                    println!("failed to find response tx for server message");
                                                }
                                            }
                                        }

                                    }
                                    Err(_) => {
                                        // TODO: Log error, unable to decode msg, report to server?
                                    }
                                }


                            } else {
                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("Time went backwards")
                                    .as_secs() as u32;
                                let _log_result = db.add_log(0, "Message recieved from the server was not binary", timestamp).await;
                                // TODO: Reconnect to server? Potential wrong as the server doesn't send this kind of message
                            }

                        }
                        Err(_) => {
                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_secs() as u32;
                            let _log_result = db.add_log(0, "Message recieved from the server was corrupt", timestamp).await;
                            // TODO: Should something else be done here? Potentially alert the server or user that the last message was corrupt
                        }
                    }
                }
            }
        }
    } else {
        println!("user is not logged in") // TOOD: handle error, this should not happen as the user should have been prompted to log in before this function is called, but just in case
    }
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
