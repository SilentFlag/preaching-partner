use crate::datatypes::{WsRequest, WsState};

use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

#[tauri::command]
pub async fn login(
    app_handle: AppHandle,
    state: State<'_, WsState>,
    _username: String,
    _password: String,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    let msg = vec![];
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
    let success = if let Ok(_msg) = response {
        // TODO: handle response
        true
    } else {
        // TODO: handle recieving message fail
        false
    };

    // TODO: Send message to frontend (webview)
    let payload = success;

    // TODO: Handle the error rather than crash with ?
    app_handle
        .emit("login-result", payload)
        .map_err(|e| e.to_string())?;

    Ok(())
}
