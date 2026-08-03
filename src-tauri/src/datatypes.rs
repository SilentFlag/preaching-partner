use futures::StreamExt;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Mutex;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

use crate::{database::MyDatabase, services};

// ------------------ MESSAGES SENT FROM CLIENT TO CLIENT ----------
pub enum FrontendReponse {
    ConfirmLogin {
        success: bool,
        name: String,
    },
    Maps(Vec<MapDisplayDetails>),
    MapDetails {
        details: MapDetails,
        streets: Vec<StreetDetails>,
        addresses: Vec<AddressDetails>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapDisplayDetails {
    pub id: u32,
    pub display_name: String,
    pub file_name: String,
    pub category: u32,
}

pub struct StartupState {
    pub request_rx: Mutex<Option<mpsc::Receiver<WsRequest>>>,
    pub event_tx: broadcast::Sender<WsEvent>,
}

pub struct WsState {
    pub request_tx: mpsc::Sender<WsRequest>,
    pub event_tx: broadcast::Sender<WsEvent>,
}

pub struct WsRequest {
    pub payload: FrontEndPayload,
    pub response_tx: oneshot::Sender<FrontendReponse>,
}

pub enum FrontEndPayload {
    Login { name: String, password: String },
    MessageForServer(ClientPayload),
    GetMapDetails(u32),
    GetMaps,
}

#[derive(Clone, Debug)]
pub struct WsEvent {
    pub payload: ServerMessage,
}

// ------------------ ABSTRACTION FOR SENDING SERVER MESSAGES ----------

pub struct WsSender {
    pub write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    pub read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WsSender {
    pub async fn send(
        &mut self,
        mut msg: ClientMessage,
        token: [u8; 32],
        db: MyDatabase,
    ) -> Result<Option<[u8; 32]>, tokio_tungstenite::tungstenite::Error> {
        let _send_attempts = 0;
        loop {
            if let Some(_token) = msg.access_token {
                let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
                // TODO: Handle error
                let _result = self
                    .write
                    .send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes))
                    .await;
                return Ok(msg.access_token);
            } else {
                let get_token_msg = ClientMessage {
                    id: 0,
                    access_token: None,
                    payload: ClientPayload::RequestAccessToken(token),
                };
                // TODO: handle errors
                let msg_bytes = rmp_serde::to_vec(&get_token_msg).unwrap();
                let _result = self
                    .write
                    .send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes))
                    .await; // ERROR DOES GO INTO
                if let Some(server_message) = self.read.next().await {
                    match server_message {
                        Ok(server_message) => {
                            if let Message::Binary(message_binary) = server_message {
                                let server_msg: Result<ServerMessage, rmp_serde::decode::Error> =
                                    rmp_serde::from_slice(&message_binary);
                                match server_msg {
                                    Ok(message) => {
                                        // TODO: Check if message is successful access token recieved, if so, save it and continue loop to try send original message again
                                        match message.payload {
                                            ServerPayload::NewAccessToken(token) => {
                                                let save_result =
                                                    services::save_access_token(db.clone(), token)
                                                        .await;
                                                match save_result {
                                                    Ok(_result) => {
                                                        msg.access_token = Some(token);
                                                        continue;
                                                    }
                                                    Err(_error) => {
                                                        // TODO: handle error
                                                    }
                                                }
                                            }
                                            _ => {
                                                // TODO: handle unexpected message
                                            }
                                        }
                                    }
                                    Err(_error) => {
                                        println!("error decoding binary"); // TODO: Handle error
                                    }
                                }
                            } else {
                                println!("unexpected message type"); // TODO: handle error
                            }
                        }
                        Err(_error) => {
                            println!("failed to get message from server"); // TODO: Handle error
                        }
                    }
                }
            }
        }
    }
}

// ------------------ MESSAGES SENT FROM CLIENT TO SERVER ----------
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub id: u32,
    pub access_token: Option<[u8; 32]>,
    pub payload: ClientPayload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientPayload {
    Login { name: String, password: String },
    UpdateCheckbox { map: i32, id: i32, checked: bool },
    UpdateCheckboxDetails { map: i32, id: i32, name: String },
    RequestAccessToken([u8; 32]),
    RequestSync(u32),
}

// ------------------ MESSAGES SENT FROM SERVER TO CLIENT ----------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerMessage {
    pub id: u32,
    pub timestamp: u32,
    pub payload: ServerPayload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerPayload {
    ConfirmLogin {
        success: bool,
        refresh_token: Option<[u8; 32]>,
        access_token: Option<[u8; 32]>,
    },
    SyncInformation(SyncInformation),
    NewAccessToken([u8; 32]),
    UnknownError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncInformation {
    pub congregations: Vec<CongDetails>,
    pub categories: Vec<CategoryDetails>,
    pub service_groups: Vec<GroupDetails>,
    pub users: Vec<UserPublicDetails>,
    pub maps: Vec<MapDetails>,
    pub streets: Vec<StreetDetails>,
    pub addresses: Vec<AddressDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CongDetails {
    pub cong_id: u32,
    pub cong_name: String,
    pub remove: bool,
    pub updated: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupDetails {
    pub id: u32,
    pub name: String,
    pub cong: u32,
    pub elder: u32,
    pub updated: u32,
    pub group_deleted: bool, // TODO: Condense into one deleted variable
    pub pair_deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapDetails {
    pub id: u32,
    pub name: String,
    pub image_name: String,
    pub assignee: u32,
    pub assigner: u32,
    pub image: Option<Vec<u8>>,
    pub category: u32,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryDetails {
    pub id: u32,
    pub name: String,
    pub prefix: String,
    pub congregation: u32,
    pub updated: u32,
    pub remove: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreetDetails {
    pub id: u32,
    pub map_id: u32,
    pub name: String,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AddressTags {
    DoNotCall,
    NoJunkMail,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressDetails {
    pub id: u32,
    pub street_id: u32,
    pub number: String,
    pub tags: Vec<AddressTags>,
    pub visited: bool,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPublicDetails {
    pub id: u32,
    pub name: String,
    pub deleted: bool,
}

/// ----------------------------- All errors relating to MyDatabase
/// // TODO: allow last errors once needed
#[derive(Debug)]
pub enum DbError {
    InvalidLocation(sqlx::Error),
    InvalidToken(u32),
    InvalidRow(sqlx::Error),
    ConnectionFailure(sqlx::Error),
    QueryFailure(sqlx::Error),
    // TokenRngFailure(SysError),
    SerialiseError(rmp_serde::encode::Error),
    AddressFailure(AddressError),
    UnknownError(sqlx::Error),
    Error,
}

impl From<rmp_serde::encode::Error> for DbError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        DbError::SerialiseError(err)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::InvalidLocation(error) => write!(f, "db not found: {}", error),
            DbError::InvalidToken(error) => {
                write!(f, "token returned an invalid number of users: {}", error)
            }
            DbError::InvalidRow(error) => {
                write!(
                    f,
                    "an invalid row was passed to a parsing function: {}",
                    error
                )
            }
            DbError::ConnectionFailure(error) => write!(f, "connection to db failed: {}", error),
            DbError::QueryFailure(error) => write!(f, "a query failed to run: {}", error),
            // DbError::TokenRngFailure(error) => write!(f, "a token failed to generate: {}", error),
            DbError::SerialiseError(error) => write!(f, "failed to serialise data: {}", error),
            DbError::AddressFailure(error) => {
                write!(f, "something went wrong with the addresses: {}", error)
            }
            DbError::UnknownError(error) => write!(f, "an unknown error occured: {}", error),
            DbError::Error => write!(f, "a dberror::error error occured"),
        }
    }
}

#[derive(Debug)]
pub enum AddressError {
    SqlxError(sqlx::Error),
    DeserialiseError(rmp_serde::decode::Error),
}

impl From<sqlx::Error> for AddressError {
    fn from(err: sqlx::Error) -> Self {
        AddressError::SqlxError(err)
    }
}

impl From<rmp_serde::decode::Error> for AddressError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        AddressError::DeserialiseError(err)
    }
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::DeserialiseError(error) => {
                write!(f, "something went wrong deserialising the tags: {}", error)
            }
            AddressError::SqlxError(error) => {
                write!(f, "something went wrong with sqlx: {}", error)
            }
        }
    }
}
