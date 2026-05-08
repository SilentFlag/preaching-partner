use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub id: u32,
    pub payload: ClientPayload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientPayload {
    Login { name: String, password: String },
    UpdateCheckbox { map: i32, id: i32, checked: bool },
    UpdateCheckboxDetails { map: i32, id: i32, name: String },
    SetLowDataMode(bool),
    RequestSync(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerMessage {
    pub id: u32,
    pub timestamp: u64,
    pub payload: ServerPayload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerPayload {
    Confirm(bool),
    ConfirmLogin {
        success: bool,
        refresh_token: Option<[u8; 32]>,
        access_token: Option<[u8; 32]>,
    },
    MapImage {
        image_name: String,
        image: Vec<u8>,
        assignee: u32,
        assigner: u32,
        category: u32,
    },
    SyncComplete,
}

#[derive(Serialize, Clone)]
pub enum FrontendReponse {
    ConfirmLogin { success: bool, name: String },
}

pub struct WsState {
    pub request_tx: mpsc::Sender<WsRequest>,
    pub event_tx: broadcast::Sender<WsEvent>,
}

pub struct WsRequest {
    pub payload: ClientPayload,
    pub response_tx: oneshot::Sender<ServerMessage>,
}

#[derive(Clone, Debug)]
pub struct WsEvent {
    pub payload: ServerMessage,
}
