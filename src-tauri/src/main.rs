// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::oneshot;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

mod datatypes;

async fn connect_to_server() {
    let url = String::from("ws://127.0.0.1:9001");

    let (ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect");

    println!("Connected to server");

    let (mut write, mut read) = ws_stream.split();

    // TODO: RUSTLS ENCRYPTION

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let message = datatypes::ClientMessage::Login {
        name: String::from("Ethan"),
        password: String::from("unset")
    };
    let message_bytes = rmp_serde::to_vec(&message).unwrap();
    write.send(tokio_tungstenite::tungstenite::Message::binary(message_bytes)).await;
    println!("Sent message");

    let msg = read.next().await;
    match msg {
        Some(msg) => {
            match msg {
                Ok(msg) => {
                    println!("Recieved message from server: {:?}", msg);
                }
                Err(error) => {
                    println!("Recieved error from server: {:?}", error);
                }
            }
        }
        None => {
            println!("Recieved Nothing from the server");
        }
    }

    write.send(tokio_tungstenite::tungstenite::Message::Close(None))
        .await
        .unwrap();
}

fn main() {
    tauri::async_runtime::spawn(async {
        connect_to_server().await;
    });

    ministry_manager_lib::run();
}
