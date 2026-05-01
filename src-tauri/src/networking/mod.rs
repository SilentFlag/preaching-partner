use crate::datatypes::{WsEvent,WsRequest,ClientMessage, ClientPayload, ServerMessage};
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::{mpsc, broadcast};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::str::FromStr;
use std::fs::File;
use std::io::Write;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use crate::{ServerPayload};

pub async fn connect_to_server(
    mut request_rx: mpsc::Receiver<WsRequest>,
    event_tx: broadcast::Sender<WsEvent>
) {

    // Create Connection
    let url = String::from("ws://127.0.0.1:9001");

    let (ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect");

    println!("Connected to server");

    let (mut write, mut read) = ws_stream.split();

    // TODO: RUSTLS ENCRYPTION

    // Open database
    let my_pool_option = SqliteConnectOptions::from_str("sqlite://../database/data.db"); // ----------------- ERROR ------------
    let conn = match my_pool_option {
        Ok(my_pool_option) => {
            let my_pool_option = my_pool_option.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
            let conn = SqlitePool::connect_with(my_pool_option).await;
            match conn {
                Ok(conn) => {
                    conn
                }
                Err(error) => {
                    panic!("Connection to database failed: {:?}", error);
                }
            }
        }
        Err(error) => {
            panic!("Database Options Failed: {:?}", error);
        }
    };
    let db = &conn;

    // TODO: Sync with server
    // let sync_success = setup::sync_with_server(&db);
    // match sync_success {
    //     Err(_) => {
    //         // TODO: Warn user of failure to sync
    //         println!("Failed to sync with server");
    //     },
    //     _ => {}
    // }


    let msg: ClientMessage = ClientMessage { // form message to send
        id: 0,
        payload: ClientPayload::RequestSync(0),
    };
    let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
    let _ = write.send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes)).await; // ERROR DOES GO INTO

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
                let _ = write.send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes)).await; // ERROR DOES GO INTO
                current_id += 1;
            }

            // Handle incoming messages
            Some(msg) = read.next() => {
                let coded_msg = msg.unwrap();
                if let Message::Binary(bin) = coded_msg {
                    let msg: ServerMessage = rmp_serde::from_slice(&bin).unwrap();
                    // Check for messages that require db writes
                    match msg.payload {
                        ServerPayload::ConfirmLogin{success: _, refresh_token, access_token} => {
                            // TODO: Handle when empty
                            // TODO: Refactor  
                            if let Some(refresh_token_ok) = refresh_token {
                                let insert_token_query = sqlx::query("INSERT INTO tokens(refresh, token) VALUES (true, ?)")
                                    .bind(hex::encode(refresh_token_ok));

                                let query_result = insert_token_query.execute(db).await;

                                if let Err(result) = query_result {
                                    // TODO: handle this error
                                    println!("Something went wrong inserting refresh token into database, error: {:?}", result)
                                }
                            }

                            if let Some(access_token_ok) = access_token {
                                let insert_token_query = sqlx::query("INSERT INTO tokens(refresh, token) VALUES (false, ?)")
                                .bind(hex::encode(access_token_ok));

                                let query_result = insert_token_query.execute(db).await;

                                if let Err(result) = query_result {
                                    // TODO: handle this error
                                    println!("Something went wrong inserting refresh token into database, error: {:?}", result)
                                }
                            }
                        }
                        ServerPayload::MapImage(ref image) => {
                            // TODO: Save image
                            println!("Recieved map image from server");
                            let new_image_file = File::create("../maps/t01.png");
                            if let Ok(mut image_file) = new_image_file {
                                let attempt_to_write = image_file.write(image);
                                if let Ok(_) = attempt_to_write {
                                    println!("Successfully saved the image");
                                }
                            } else {
                                println!("Failed to create image file");
                            }
                        }
                        _ => {
                            // TODO: Ignore?
                        }
                    }

                    // Send response back to original caller
                    if msg.id == 0 {
                        let _ = event_tx.send(WsEvent { payload: msg });
                    } else {
                        let response_tx = response_senders.remove(&msg.id);
                        match response_tx {
                            Some(response_tx) => {
                                let _ = response_tx.send(msg);
                            }
                            _ => {
                                println!("failed to find response tx for server message");
                            }
                        }
                    }
                }
            }
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
