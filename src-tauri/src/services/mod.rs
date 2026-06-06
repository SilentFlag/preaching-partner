use crate::{database::MyDatabase, datatypes::DbError};

pub async fn save_access_token(db: MyDatabase, token: [u8; 32]) -> Result<(), DbError> {
    db.save_access_token(token).await
}

pub async fn save_refresh_token(db: MyDatabase, token: [u8; 32]) -> Result<(), DbError> {
    db.save_refresh_token(token).await
}
