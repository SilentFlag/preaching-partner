use crate::datatypes::{FrontendReponse, WsRequest, WsState};
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
    let msg = crate::datatypes::FrontEndPayload::Login {
        name: username,
        password,
    };
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

    let response: Result<FrontendReponse, String> = rx.await.map_err(|e| e.to_string());
    // TODO: handle error case
    let success = if let Ok(msg) = response {
        match msg {
            FrontendReponse::ConfirmLogin { success, .. } => success,
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

    let emission_result = app_handle.emit("login", success).map_err(|e| e.to_string());
    if let Err(error) = emission_result {
        println!(
            "There was an error sending a message to the frontend: {}",
            error
        );
    }

    Ok(())
}
