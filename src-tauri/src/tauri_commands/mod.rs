use std::vec;

use crate::core;
use crate::datatypes::{FrontEndPayload, FrontendReponse, StartupState};
use crate::datatypes::{WsRequest, WsState};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

/// Called when the frontend has loaded.
/// Extracts sender and receiver from state and calls core::initiate_backend() with them
///
/// Parameters:
///     state: State
///     app_handle: AppHandle
///
/// Return Value:
///     Ok(())
///     Err(())
///
/// TODO: check if backend successfully started, if not, inform user with error
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

/// Called when the frontend wants the data of all maps
///
/// Return Value:
///     Returns the map data to the frontend in the form of FrontendReponse::Maps
///
/// Errors:
/// Sending message to the networking
/// Invalid response from networking
/// Sending message to the frontend
#[tauri::command]
pub async fn get_maps(app: AppHandle, state: State<'_, WsState>) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    let msg = FrontEndPayload::GetMaps;
    let request = WsRequest {
        payload: msg,
        response_tx: tx,
    };

    // TODO: handle error
    let _sent_mess = state
        .request_tx
        .send(request)
        .await
        .map_err(|e| e.to_string());

    let response: Result<FrontendReponse, String> = rx.await.map_err(|e| e.to_string());
    // TODO: handle error case
    let maps = if let Ok(msg) = response {
        match msg {
            FrontendReponse::Maps(maps) => maps,
            _ => {
                // TODO: Handle this case, it should never be reached
                println!("Unexpected message from server");
                vec![]
            }
        }
    } else {
        // TODO: handle recieving message fail
        vec![]
    };

    let emission_result = app.emit("maps", maps).map_err(|e| e.to_string());
    if let Err(error) = emission_result {
        // TODO: handle error
        println!(
            "There was an error sending a message to the frontend: {}",
            error
        );
    }

    Ok(())
}
