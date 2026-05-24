use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientAction {
    CheckBox,
    AssignMap,
    AddUser,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientMessage {
    pub id: u32,
    pub action_list: Vec<ClientAction>,
    pub name: String,
    pub token: Vec<u8>,
    pub payload: Vec<u8>,
}

pub struct WsState {
    pub request_tx: mpsc::Sender<WsRequest>,
    pub event_tx: broadcast::Sender<WsEvent>,
}

pub struct WsRequest {
    pub payload: Vec<u8>,
    pub response_tx: oneshot::Sender<ClientMessage>,
}

#[derive(Clone, Debug)]
pub struct WsEvent {
    pub payload: ClientMessage,
}
