use crate::core;
use crate::datatypes::{ClientPayload, ServerPayload, StartupState, WsRequest, WsState};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

#[tauri::command]
pub async fn app_loaded(state: State<'_, StartupState>, app_handle: AppHandle) -> Result<(), ()> {
    let request_rx = {
        let mut guard = state.request_rx.lock().unwrap();
        guard.take()
    };
    let event_tx = state.event_tx.clone();

    if let Some(request_rx) = request_rx {
        tauri::async_runtime::spawn(async {
            core::initiate_backend(request_rx, event_tx, app_handle).await;
        });
    }

    Ok(())
}

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

    let emission_result = app_handle.emit("login", success).map_err(|e| e.to_string());
    if let Err(error) = emission_result {
        println!(
            "There was an error sending a message to the frontend: {}",
            error
        );
    }

    Ok(())
}
