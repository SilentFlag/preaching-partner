// TODO: Write code that loads everything, eg check database for tokens then request new refresh token

use crate::database::MyDatabase;
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

/// function to call for a full sync with the server
pub async fn sync_with_server(
    db: MyDatabase,
    write: &mut WsSink,
    read: &mut WsSource,
) -> Result<bool, bool> {
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

    // Loop until recieve a completed sync message

    loop {
        if let Some(msg) = read.next().await {
            let msg: tokio_tungstenite::tungstenite::Message = msg.unwrap();
            if let Message::Binary(bin) = msg {
                let msg: ServerMessage = rmp_serde::from_slice(&bin).unwrap();
                let timestamp = msg.timestamp;
                let timestamp_vec = rmp_serde::to_vec(&timestamp);
                match timestamp_vec {
                    Ok(timestamp_vec) => {
                        // Check for messages that require db writes
                        match msg.payload {
                            ServerPayload::SyncInformation(sync_info) => {
                                let congregations = sync_info.congregations;
                                let categories = sync_info.categories;
                                let service_groups = sync_info.service_groups;
                                let _users = sync_info.users;

                                // TODO: Error Handling
                                for cong in congregations {
                                    let _ = db.update_congregation(cong);
                                }
                                for category in categories {
                                    let _ = db.update_category(category);
                                }
                                for service_group in service_groups {
                                    let _ = db.update_service_group(service_group);
                                }
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
