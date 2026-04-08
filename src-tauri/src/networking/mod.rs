use crate::datatypes::{WsEvent,WsRequest,ClientMessage};
use tokio::sync::{mpsc, broadcast};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

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
