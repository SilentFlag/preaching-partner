use crate::database::MyDatabase;
use crate::datatypes::{
    ClientMessage, ClientPayload, FrontEndPayload, FrontendReponse, ServerMessage, ServerPayload,
    WsEvent, WsRequest,
};
use crate::networking;
use crate::services::is_logged_in;
use crate::services::{save_access_token, save_refresh_token};
use reqwest::Client as HttpClient;
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, mpsc};

/// Start running backend code once the frontend has loaded
/// Open the database and check if the user is logged in:
/// TRUE:
///     if so, run networking::connect_to_server()
///
/// False:
///     Wait for a login attempt from the frontend, attempt to login, check if successful:
///         TRUE: run networking::connect_to_server()
///         FALSE: repeat
///
/// TODO: Error handling
///
pub async fn initiate_backend(
    mut request_rx: mpsc::Receiver<WsRequest>,
    event_tx: broadcast::Sender<WsEvent>,
    app_handle: AppHandle,
) {
    // TODO: RUSTLS ENCRYPTION

    // Open database
    let db = MyDatabase::new().await;
    match db {
        Ok(db) => {
            // TODO: Check if logged in
            let logged_in = is_logged_in(db.clone()).await;

            match logged_in {
                Some(token) => {
                    println!("User is logged in with refresh token: {:?}", token);
                    // TODO: Handle result
                    let emission_result = app_handle.emit("login", true).map_err(|e| e.to_string());
                    if let Err(error) = emission_result {
                        println!(
                            "There was an error sending a message to the frontend: {}",
                            error
                        );
                    }
                    networking::connect_to_server(request_rx, event_tx, db, app_handle).await;
                }
                None => {
                    println!("User is not logged in");
                    loop {
                        // wait for incoming login request from the frontend
                        if let Some(req) = request_rx.recv().await {
                            let client_payload: FrontEndPayload = req.payload.into(); // extract payload
                            let response_sender = req.response_tx;

                            match client_payload {
                                FrontEndPayload::Login { name, password } => {
                                    // form message to send
                                    let msg: ClientMessage = ClientMessage {
                                        id: 0,
                                        access_token: None,
                                        payload: ClientPayload::Login {
                                            name: name.clone(),
                                            password,
                                        },
                                    };
                                    let msg_bytes: Vec<u8> = rmp_serde::to_vec(&msg).unwrap();

                                    // Send message to server
                                    let client = HttpClient::new();
                                    let login_response = client
                                        .post("http://localhost:9001/login")
                                        .header("Content-Type", "application/octet-stream")
                                        .body(msg_bytes)
                                        .send()
                                        .await;

                                    // handle response from server
                                    match login_response {
                                        Ok(response) => {
                                            let response_bytes = response.bytes().await;
                                            match response_bytes {
                                                Ok(bytes) => {
                                                    let response_msg: Result<
                                                        ServerMessage,
                                                        rmp_serde::decode::Error,
                                                    > = rmp_serde::from_slice(&bytes);
                                                    match response_msg {
                                                        Ok(msg) => {
                                                            println!(
                                                                "Received login response: {:?}",
                                                                msg
                                                            );
                                                            match msg.payload {
                                                                ServerPayload::ConfirmLogin {
                                                                    success,
                                                                    refresh_token,
                                                                    access_token,
                                                                } => {
                                                                    if success {
                                                                        println!(
                                                                            "login  successful"
                                                                        );
                                                                        // Login successful
                                                                        if let Some(
                                                                            refresh_token_ok,
                                                                        ) = refresh_token
                                                                        {
                                                                            let _refresh_result = save_refresh_token(db.clone(), refresh_token_ok).await;
                                                                        } else {
                                                                            println!("Login failed: No refresh token provided");
                                                                            // TODO: Send alert to frontend
                                                                            // TODO: Handle error, this shouldn't be reached
                                                                        }

                                                                        if let Some(
                                                                            access_token_ok,
                                                                        ) = access_token
                                                                        {
                                                                            let _access_token_result =
                                                                                save_access_token(
                                                                                    db.clone(),
                                                                                    access_token_ok,
                                                                                )
                                                                                .await;
                                                                        } else {
                                                                            println!("Login failed: No access token provided");
                                                                            // TODO: Send alert to frontend
                                                                            // TODO: Handle error, this shouldn't be reached
                                                                        }

                                                                        // Inform frontend of login
                                                                        // TODO: Handle error
                                                                        let response = FrontendReponse::ConfirmLogin { success, name };
                                                                        let _send_response_result =
                                                                            response_sender
                                                                                .send(response);

                                                                        // Now that we're logged in, connect to the server websocket
                                                                        networking::connect_to_server(request_rx, event_tx, db.clone(), app_handle).await;
                                                                        // Break because login was successful and we don't want to keep waiting for login requests
                                                                        break;
                                                                    } else {
                                                                        // TODO: Handle error
                                                                        let response = FrontendReponse::ConfirmLogin { success, name };
                                                                        let _send_response_result =
                                                                            response_sender
                                                                                .send(response);
                                                                    }
                                                                }
                                                                _ => {
                                                                    println!("Unexpected login response payload: {:?}", msg.payload);
                                                                }
                                                            }
                                                        }
                                                        Err(error) => {
                                                            println!(
                                                                "Failed to decode login response: {:?}",
                                                                error
                                                            ); // TODO: Handle and log error
                                                        }
                                                    }
                                                }
                                                Err(error) => {
                                                    println!(
                                                        "Failed to read login response bytes: {:?}",
                                                        error
                                                    ); // TODO: Handle and log error
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            println!("Login request failed: {:?}", error);
                                            // TODO: Handle and log error
                                        }
                                    }
                                }
                                _ => {
                                    // Unexpected message, ignore, TODO: log this
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            // TODO: Alert user of being unable to connect to database
            panic!(
                "Something went wrong trying to connect to the database: {}",
                error
            );
        }
    }
}
