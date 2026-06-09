use crate::database::MyDatabase;
use crate::datatypes::{
    ClientMessage, ClientPayload, ServerMessage, ServerPayload, WsEvent, WsRequest,
};
use crate::services::{save_access_token, save_refresh_token};
use crate::sync::sync_with_server;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn connect_to_server(
    mut request_rx: mpsc::Receiver<WsRequest>,
    event_tx: broadcast::Sender<WsEvent>,
) {
    // Create Connection
    let url = String::from("ws://127.0.0.1:9001");

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

    println!("Connected to server");

    let (mut write, mut read) = ws_stream.split();

    // TODO: RUSTLS ENCRYPTION

    // Open database
    let db = MyDatabase::new().await;
    match db {
        Ok(db) => {
            // Request to sync after initial setup of db and connection has been established
            // TODO: Set the time to the last time it has synced rather than a concrete value
            let msg: ClientMessage = ClientMessage {
                // form message to send
                id: 0,
                payload: ClientPayload::RequestSync(0),
            };
            let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
            let _ = write
                .send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes))
                .await; // ERROR DOES GO INTO

            let mut current_id: u32 = 1; // Message id, don't start at 0, 0 indicates global message
            let mut response_senders = HashMap::new();

            // Core loop

            loop {
                tokio::select! {
                    // handle io messages
                    Some(req) = request_rx.recv() => {
                        // TODO: Check database for existing data before sending request

                        // handle outgoing ws message
                        response_senders.insert(current_id, req.response_tx); // remember reponse_tx for later in hashmap
                        let client_payload: ClientPayload = req.payload.into(); // extract payload

                        let msg: ClientMessage = ClientMessage { // form message to send
                            id: current_id,
                            payload: client_payload,
                        };
                        let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
                        let _send_message_result = write.send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes)).await; // ERROR DOES GO INTO
                        current_id += 1;

                        println!("Sent server message")
                    }

                    // Handle incoming messages
                    Some(msg) = read.next() => {
                        let coded_msg = msg.expect("Something went wrong reading the next message from the server");
                        println!("read message from server");
                        if let Message::Binary(bin) = coded_msg {
                            let msg: ServerMessage = rmp_serde::from_slice(&bin).expect("Something went wrong decoding message");

                            let response_msg = msg.clone();
                            println!("\n Response from server: {:?} \n", msg.clone());

                            let timestamp = msg.timestamp;
                            let timestamp_vec = rmp_serde::to_vec(&timestamp);
                            match timestamp_vec {
                                Ok(_timestamp_vec) => {

                                // Check for messages that require db writes
                                // TODO: unsure why the message being unknown_error crashes it
                                match msg.payload {
                                    ServerPayload::ConfirmLogin{success: _, refresh_token, access_token} => {

                                        // TODO: Handle Errors
                                        if let Some(refresh_token_ok) = refresh_token {
                                            let _refresh_result = save_refresh_token(db.clone(), refresh_token_ok).await;
                                        }

                                        if let Some(access_token_ok) = access_token {
                                            let _access_token_result = save_access_token(db.clone(), access_token_ok).await;
                                        }
                                    }
                                    ServerPayload::SyncInformation(sync_info) => {
                                        let sync_result = sync_with_server(db.clone(), sync_info).await;
                                        if let Err(_error) = sync_result {
                                            // TODO: handle error
                                        }
                                    }
                                    _ => {
                                        // TODO: handle unexpected message
                                        continue;
                                    }
                                    // Send response back to original caller
                                }

                                if msg.id == 0 {
                                    let send_results = event_tx.send(WsEvent { payload: response_msg });
                                    println!("failed to send message: {:?}", send_results);
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
                                    // TODO: Handle this error
                                    println!("failed to manage timestamp")
                                }
                            }
                        } else {
                            println!("Message was not binary");
                        }
                    }
                }
            }
        }
        Err(error) => {
            panic!(
                "Something went wrong trying to connect to the database: {}",
                error
            );
        }
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
