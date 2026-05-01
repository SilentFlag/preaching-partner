// TODO: Write code that loads everything, eg check database for tokens then request new refresh token

async fn sync_with_server(_db: &sqlx::Pool<sqlx::Sqlite>) -> Result<bool, bool> {
    // TODO: sync

    // let msg: ClientMessage = ClientMessage { // form message to send
    //     id: 0,
    //     payload: ClientPayload::RequestSync(0),
    // };
    // let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
    // let _ = write.send(tokio_tungstenite::tungstenite::Message::binary(msg_bytes)).await; // ERROR DOES GO INTO

    Ok(true)
}