pub mod datatypes;
pub mod user_input;

use datatypes::{WsEvent, WsRequest, WsState};
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc};

use tokio_tungstenite::connect_async;

async fn core_loop(
    mut request_rx: mpsc::Receiver<WsRequest>,
    _event_tx: broadcast::Sender<WsEvent>,
) {
    // Create Connection
    let url = String::from("ws://127.0.0.1:9001");

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

    println!("Connected to server");

    let (mut _write, mut read) = ws_stream.split();

    // TODO: RUSTLS ENCRYPTION

    // Open database
    // let my_pool_option = SqliteConnectOptions::from_str("sqlite://../database/data.db"); // ----------------- ERROR ------------
    // let conn = match my_pool_option {
    //     Ok(my_pool_option) => {
    //         let my_pool_option = my_pool_option.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    //         let conn = SqlitePool::connect_with(my_pool_option).await;
    //         match conn {
    //             Ok(conn) => conn,
    //             Err(error) => {
    //                 panic!("Connection to database failed: {:?}", error);
    //             }
    //         }
    //     }
    //     Err(error) => {
    //         panic!("Database Options Failed: {:?}", error);
    //     }
    // };
    // let db = &conn;

    // Loop to check for messages and stuff
    loop {
        tokio::select! {
            // handle io messages
            Some(_req) = request_rx.recv() => {
                // TODO: Check database for existing data before sending request

                // handle outgoing ws message

            }

            // Handle incoming messages
            Some(_msg) = read.next() => {
                // TODO: messages from the server
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (request_tx, request_rx) = mpsc::channel(32);
    let (event_tx, _) = broadcast::channel(32);

    let ws_state = WsState {
        request_tx,
        event_tx: event_tx.clone(),
    };

    // Spawn backend loop
    tauri::async_runtime::spawn(async {
        core_loop(request_rx, event_tx).await;
    });

    tauri::Builder::default()
        .manage(ws_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![user_input::login])
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
