use crate::datatypes::{CategoryDetails, CongDetails, DbError, GroupDetails};
use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqliteRow, Pool, Row, Sqlite, SqlitePool};
use std::str::FromStr;

#[derive(Clone)]
pub struct MyDatabase {
    data: Pool<Sqlite>,
}

impl MyDatabase {
    /// Create new connection to the database
    pub async fn new() -> Result<Self, DbError> {
        let my_pool_option = SqliteConnectOptions::from_str("sqlite://../database/data.db");
        let conn = match my_pool_option {
            Ok(my_pool_option) => {
                let my_pool_option =
                    my_pool_option.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
                let conn = SqlitePool::connect_with(my_pool_option).await;
                match conn {
                    Ok(conn) => conn,
                    Err(error) => return Err(DbError::ConnectionFailure(error)),
                }
            }
            Err(error) => return Err(DbError::InvalidLocation(error)),
        };
        Ok(MyDatabase { data: conn })
    }

    // ------------------- SAVE TOKENS -------------

    pub async fn save_access_token(&self, token: [u8; 32]) -> Result<(), DbError> {
        let insert_token_query =
            sqlx::query("INSERT INTO tokens(refresh, token) VALUES (false, ?)")
                .bind(hex::encode(token));

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            // TODO: handle this error
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn save_refresh_token(&self, token: [u8; 32]) -> Result<(), DbError> {
        let insert_token_query = sqlx::query("INSERT INTO tokens(refresh, token) VALUES (true, ?)")
            .bind(hex::encode(token));

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            // TODO: handle this error
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- CONGREGATIONS -------------

    pub async fn get_congregation(&self, id: u32) -> Result<CongDetails, DbError> {
        let query = sqlx::query("SELECT * FROM congregation WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let cong_details = cong_row_to_details(&rows[0]);
                    match cong_details {
                        Ok(details) => return Ok(details),
                        Err(error) => return Err(DbError::InvalidRow(error)),
                    }
                    // return Ok(user_id);
                } else {
                    return Err(DbError::InvalidToken(rows.len() as u32));
                }
            }
            Err(error) => return Err(DbError::QueryFailure(error)),
        }
    }

    pub async fn add_congregation(&self, cong: CongDetails) -> Result<(), DbError> {
        let updated_vec = rmp_serde::to_vec(&cong.updated).expect("TODO: Handle this error");
        let insert_token_query =
            sqlx::query("INSERT INTO congregation(id, name, udpated) VALUES (?, ?, ?)")
                .bind(cong.cong_id)
                .bind(cong.cong_name)
                .bind(hex::encode(hex::encode(updated_vec)));

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_congregation(&self, cong: CongDetails) -> Result<(), DbError> {
        let insert_token_query = if cong.remove {
            sqlx::query("DELETE FROM congregation WHERE id = ?").bind(cong.cong_id)
        } else {
            let updated_vec = rmp_serde::to_vec(&cong.updated).expect("TODO: Handle this error");
            sqlx::query("UPDATE congregation SET name = ?, updated = ? WHERE id = ?")
                .bind(cong.cong_name)
                .bind(hex::encode(hex::encode(updated_vec)))
                .bind(cong.cong_id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- CATEGORIES -------------

    pub async fn get_category(&self, id: u32) -> Result<CategoryDetails, DbError> {
        let query = sqlx::query("SELECT * FROM categories WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let cong_details = category_row_to_details(&rows[0]);
                    match cong_details {
                        Ok(details) => return Ok(details),
                        Err(error) => return Err(DbError::InvalidRow(error)),
                    }
                    // return Ok(user_id);
                } else {
                    return Err(DbError::InvalidToken(rows.len() as u32));
                }
            }
            Err(error) => return Err(DbError::QueryFailure(error)),
        }
    }

    pub async fn add_category(&self, category: CategoryDetails) -> Result<(), DbError> {
        let updated_vec = rmp_serde::to_vec(&category.updated).expect("TODO: Handle this error");
        let insert_token_query =
            sqlx::query("INSERT INTO categories(id, name, prefix, congregation, updated) VALUES (?, ?, ?, ?, ?)")
                .bind(category.id)
                .bind(category.name)
                .bind(category.prefix)
                .bind(category.congregation)
                .bind(hex::encode(updated_vec));

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_category(&self, category: CategoryDetails) -> Result<(), DbError> {
        let insert_token_query = if category.remove {
            sqlx::query("DELETE FROM congregation WHERE id = ?").bind(category.id)
        } else {
            let updated_vec =
                rmp_serde::to_vec(&&category.updated).expect("TODO: Handle this error");
            sqlx::query("UPDATE categories SET name = ?, prefix = ?, congregation = ?, updated = ? WHERE id = ?")
                .bind(category.name)
                .bind(category.prefix)
                .bind(category.congregation)
                .bind(hex::encode(hex::encode(updated_vec)))
                .bind(category.id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- SERVICE GROUPS -------------

    pub async fn get_service_group(&self, id: u32) -> Result<GroupDetails, DbError> {
        let query = sqlx::query("SELECT * FROM service_group WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let cong_details = group_row_to_details(&rows[0]);
                    match cong_details {
                        Ok(details) => return Ok(details),
                        Err(error) => return Err(DbError::InvalidRow(error)),
                    }
                    // return Ok(user_id);
                } else {
                    return Err(DbError::InvalidToken(rows.len() as u32));
                }
            }
            Err(error) => return Err(DbError::QueryFailure(error)),
        }
    }

    pub async fn add_service_group(&self, details: GroupDetails) -> Result<(), DbError> {
        let updated_vec = rmp_serde::to_vec(&details.updated).expect("TODO: Handle this error");
        let insert_token_query =
            sqlx::query("INSERT INTO service_group(id, name, congregation, elder, updated) VALUES (?, ?, ?, ?, ?)")
                .bind(details.id)
                .bind(details.name)
                .bind(details.cong)
                .bind(details.elder)
                .bind(hex::encode(updated_vec));

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_service_group(&self, details: GroupDetails) -> Result<(), DbError> {
        let insert_token_query = if details.group_deleted || details.pair_deleted {
            sqlx::query("DELETE FROM service_group WHERE id = ?").bind(details.id)
        } else {
            let updated_vec = rmp_serde::to_vec(&details.updated).expect("TODO: Handle this error");
            sqlx::query("UPDATE service_group SET name = ?, congregation = ?, elder = ?, updated = ? WHERE id = ?")
                .bind(details.name)
                .bind(details.cong)
                .bind(details.elder)
                .bind(hex::encode(hex::encode(updated_vec)))
                .bind(details.id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- MAPS -------------

    // ------------------- USERS -------------

    // ------------------- ADDRESSES -------------
}

fn cong_row_to_details(row: &SqliteRow) -> Result<CongDetails, sqlx::Error> {
    let cong_id: u32 = row.try_get("congregation_id")?;
    let cong_name: String = row.try_get("name")?;
    let updated: u64 = row.try_get("updated")?;
    Ok(CongDetails {
        cong_id,
        cong_name,
        remove: false,
        updated,
    })
}

/// Given a SqliteRow, return the details of the category
///
/// Parameter:
///     row: A SqliteRow of the categories table
///
/// Return Value:
///     Ok(MapDetails): Category details from row returned when successful
///     Err(sqlx::Error): Error when getting the collumns, caused by row from the wrong table
///
/// TODO: does CategoryDetails need a timestamp field?
fn category_row_to_details(row: &SqliteRow) -> Result<CategoryDetails, sqlx::Error> {
    let id = row.try_get("id")?;
    let name = row.try_get("name")?;
    let prefix = row.try_get("prefix")?;
    let congregation = row.try_get("congregation")?;
    let updated = row.try_get("updated")?;
    Ok(CategoryDetails {
        id,
        name,
        prefix,
        congregation,
        updated,
        remove: false,
    })
}

/// Given a SqliteRow of a groups details, return the formatted details
///
/// Query for rows: "SELECT user_group_pair.group_id AS group_id, user_group_pair.deleted AS pair_deleted, user_group_pair.updated AS pair_updated, service_group.name AS name, service_group.elder AS elder, service_group.deleted AS group_deleted, service_group.updated AS group_updated, service_group.congregation AS congregation  FROM user_group_pair INNER JOIN service_group ON service_group.id=user_group_pair.group_id WHERE user_id = ?"
///
/// Parameters:
///     row: SqliteRow of the group details
///
/// Return Value:
///     Ok(GroupDetails): Function successful
///     Err(sqlx::Error): Sqlx Error occured
fn group_row_to_details(row: &SqliteRow) -> Result<GroupDetails, sqlx::Error> {
    let id = row.try_get("group_id")?;
    let name: String = row.try_get("name")?;
    let cong: u32 = row.try_get("congregation")?;
    let elder: u32 = row.try_get("elder")?;
    let group_updated: u64 = row.try_get("group_updated")?;
    let pair_updated: u64 = row.try_get("pair_updated")?;
    let updated: u64 = if group_updated > pair_updated {
        group_updated
    } else {
        pair_updated
    };
    let group_deleted: bool = row.try_get("delted")?;
    let pair_deleted: bool = row.try_get("pair_delted")?;
    Ok(GroupDetails {
        id,
        name,
        cong,
        elder,
        updated,
        group_deleted,
        pair_deleted,
    })
}
