use crate::datatypes::{WsEvent,WsRequest,ClientMessage, ClientPayload, ServerMessage};
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::{mpsc, broadcast};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;

pub async fn connect_to_server(
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

    let mut current_id: u32 = 1; // Don't start at 0, 0 indicates global message
    let mut response_senders = HashMap::new();

    loop {
        tokio::select! {
            Some(req) = request_rx.recv() => {
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

            // 🔹 Handle incoming unsolicited messages
            Some(msg) = read.next() => {
                let coded_msg = msg.unwrap();
                if let Message::Binary(bin) = coded_msg {
                    let msg: ServerMessage = rmp_serde::from_slice(&bin).unwrap();
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
