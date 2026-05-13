use crate::datatypes::{ClientPayload, FrontendReponse, ServerPayload, WsRequest, WsState};

use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

#[tauri::command]
pub async fn login(
    app_handle: AppHandle,
    state: State<'_, WsState>,
    username: String,
    password: String,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    let msg = ClientPayload::Login {
        name: username,
        password: password,
    };
    println!("{:?}", &msg);
    let request = WsRequest {
        payload: msg,
        response_tx: tx,
    };

    let sent_mess = state
        .request_tx
        .send(request)
        .await
        .map_err(|e| e.to_string());
    println!("message sent something {:?}", sent_mess);

    let response = rx.await.map_err(|e| e.to_string());
    println!("Recieved message to login function: {:?}", response);
    // TODO: handle error case
    let success = if let Ok(msg) = response {
        match msg.payload {
            ServerPayload::ConfirmLogin { success, .. } => success,
            _ => {
                // TODO: Handle this case
                println!("Unexpected message from server");
                false
            }
        }
    } else {
        // TODO: handle recieving message fail
        false
    };

    // TODO: Send message to frontend (webview)
    let payload = FrontendReponse::ConfirmLogin {
        success,
        name: "Default".to_string(), // or Some(token_string)
    };

    // TODO: Handle the error rather than crash with ?
    app_handle
        .emit("login-result", payload)
        .map_err(|e| e.to_string())?;

    Ok(())
}
