// TODO: Write code that loads everything, eg check database for tokens then request new refresh token

use crate::datatypes::{ClientMessage, ClientPayload, ServerMessage, ServerPayload};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::fs::File;
use std::io::Write;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsSource = SplitStream<WsStream>;

pub async fn sync_with_server(
    db: &sqlx::Pool<sqlx::Sqlite>,
    write: &mut WsSink,
    read: &mut WsSource,
) -> Result<bool, bool> {
    // TODO: sync

    let msg: ClientMessage = ClientMessage {
        // form message to send
        id: 0,
        payload: ClientPayload::RequestSync(0),
    };
    let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
    let _ = write
        .send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes))
        .await; // ERROR DOES GO INTO

    // Loop until recieve a completed sync message

    loop {
        tokio::select! {
            // Handle incoming messages
            Some(msg) = read.next() => {
                let coded_msg = msg.unwrap();
                if let Message::Binary(bin) = coded_msg {
                    let msg: ServerMessage = rmp_serde::from_slice(&bin).unwrap();
                    let timestamp = msg.timestamp;
                    let timestamp_vec = rmp_serde::to_vec(&timestamp);
                    match timestamp_vec {
                        Ok(timestamp_vec) => {

                            // Check for messages that require db writes
                    match msg.payload {
                        ServerPayload::MapImage{..} => {

                            let _ = save_image(msg.payload, timestamp_vec, db);

                        }
                        ServerPayload::SyncComplete => {
                            break;
                        }
                        _ => {
                            // TODO: Unexpected message, Ignore?
                        }
                    }

                        }
                        Err(_) => {
                            // TODO: Handle error
                        }
                    }
                }
            }
        }
    }

    Ok(true)
}

pub async fn save_image(
    image_payload: ServerPayload,
    timestamp: Vec<u8>,
    db: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), ()> {
    match image_payload {
        ServerPayload::MapImage {
            image_name,
            image,
            assignee,
            assigner,
            category,
        } => {
            let new_image_file = File::create(format!("../maps/{}", image_name.as_str()));
            if let Ok(mut image_file) = new_image_file {
                let attempt_to_write = image_file.write(&image);
                if let Ok(_) = attempt_to_write {
                    println!("Successfully saved the image");
                }
            } else {
                println!("Failed to create image file");
            }

            // TODO: Save to database

            let insert_image_query = sqlx::query(
        "INSERT INTO maps(assignee, assigner, category, file_name, updated) VALUES (?,?,?,?,?,?)",
    )
    .bind(assignee)
    .bind(assigner)
    .bind(category)
    .bind(hex::encode(image_name))
    .bind(hex::encode(timestamp));

            let query_result = insert_image_query.execute(db).await;

            if let Err(result) = query_result {
                // TODO: handle this error
                println!(
                    "Something went wrong inserting refresh token into database, error: {:?}",
                    result
                )
            }
            return Ok(());
        }
        _ => {
            return Err(());
        }
    }
}
