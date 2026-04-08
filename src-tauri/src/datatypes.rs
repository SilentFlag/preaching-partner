// use serde::{Serialize, Deserialize};

pub mod datatypes {
    use serde::{Serialize, Deserialize};
    use tokio::sync::{mpsc, oneshot, broadcast};

    #[derive(Serialize, Deserialize, Debug)]
    pub enum ClientMessage {
        Login {name: String, password: String},
        UpdateCheckbox {map: i32, id: i32, checked: bool},
        UpdateCheckboxDetails {map: i32, id: i32, name: String},
        SetLowDataMode (bool)
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum ServerMessage {
        Confirm (bool)
    }

    pub struct WsState {
        pub request_tx: mpsc::Sender<WsRequest>,
        pub event_tx: broadcast::Sender<WsEvent>,
    }

    pub struct WsRequest {
        pub payload: String,
        pub response_tx: oneshot::Sender<String>,
    }

    #[derive(Clone, Debug)]
    pub struct WsEvent {
        pub payload: String,
    }
}