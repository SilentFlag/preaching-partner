use crate::{database::MyDatabase, datatypes::DbError};

pub async fn save_access_token(db: MyDatabase, token: [u8; 32]) -> Result<(), DbError> {
    db.save_access_token(token).await
}

pub async fn save_refresh_token(db: MyDatabase, token: [u8; 32]) -> Result<(), DbError> {
    db.save_refresh_token(token).await
}

pub async fn is_logged_in(db: MyDatabase) -> Option<[u8; 32]> {
    let logged_in = db.get_refresh_token().await;
    match logged_in {
        Ok(token) => Some(token),
        Err(error) => match error {
            DbError::InvalidToken(_) => None,
            _ => None, // TODO: Log error with database
        },
    }
}

pub async fn get_access_token(db: MyDatabase) -> Option<[u8; 32]> {
    let access_token = db.get_access_token().await;
    match access_token {
        Ok(token) => Some(token),
        Err(error) => match error {
            DbError::InvalidToken(_) => None,
            _ => None, // TODO: Log error with database
        },
    }
}

pub async fn check_address(db: MyDatabase, id: u32, checked: bool) -> Result<(), DbError> {
    db.update_address_checked(id, checked).await
}
