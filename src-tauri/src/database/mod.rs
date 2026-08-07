use crate::datatypes::{
    AddressDetails, AddressError, AddressTags, CategoryDetails, CongDetails, DbError, GroupDetails,
    MapDetails, MapDisplayDetails, StreetDetails, UserPublicDetails,
};
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

    pub async fn get_refresh_token(&self) -> Result<[u8; 32], DbError> {
        let get_token_query = sqlx::query("SELECT token FROM tokens WHERE refresh = true");

        let query_result = get_token_query.fetch_all(&self.data).await;

        match query_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let token = token_row_to_details(&rows[0]);
                    match token {
                        Ok(token) => Ok(token),
                        Err(error) => return Err(DbError::InvalidRow(error)),
                    }
                } else {
                    return Err(DbError::InvalidToken(rows.len() as u32));
                }
            }
            Err(result) => {
                return Err(DbError::QueryFailure(result));
            }
        }
    }

    pub async fn get_access_token(&self) -> Result<[u8; 32], DbError> {
        let get_token_query = sqlx::query("SELECT token FROM tokens WHERE refresh = false");

        let query_result = get_token_query.fetch_all(&self.data).await;

        match query_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let token = token_row_to_details(&rows[0]);
                    match token {
                        Ok(token) => Ok(token),
                        Err(error) => return Err(DbError::InvalidRow(error)),
                    }
                } else {
                    return Err(DbError::InvalidToken(rows.len() as u32));
                }
            }
            Err(result) => {
                return Err(DbError::QueryFailure(result));
            }
        }
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
            sqlx::query("INSERT INTO congregation(id, name, updated) VALUES (?, ?, ?)")
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
        .bind(category.name.clone())
        .bind(category.prefix.clone())
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

    pub async fn get_maps(&self) -> Result<Vec<MapDisplayDetails>, DbError> {
        let query =
            sqlx::query("SELECT maps.id AS id, maps.name AS name, maps.category AS category, maps.file_name AS file_name, categories.prefix AS prefix FROM maps INNER JOIN categories ON maps.category=categories.id");

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                let mut maps = vec![];
                for map in rows {
                    let map_details_result = map_row_to_display_details(&map);
                    match map_details_result {
                        Ok(details) => maps.push(details),
                        Err(_error) => {
                            // TODO: handle error
                        }
                    }
                }
                Ok(maps)
            }
            Err(error) => return Err(DbError::QueryFailure(error)),
        }
    }

    pub async fn get_map(&self, id: u32) -> Result<MapDetails, DbError> {
        let query = sqlx::query("SELECT * FROM maps WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let user_details_result = map_row_to_details(&rows[0]);
                    match user_details_result {
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

    pub async fn add_map(
        &self,
        MapDetails {
            id,
            name,
            image_name,
            assignee,
            assigner,
            image: _,
            category,
            deleted: _,
        }: &MapDetails,
    ) -> Result<(), DbError> {
        let insert_token_query = sqlx::query(
            "INSERT INTO maps(id, name, assignee, assigner, category, file_name) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(assignee)
        .bind(assigner)
        .bind(category)
        .bind(image_name);

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_map(
        &self,
        MapDetails {
            id,
            name,
            image_name,
            assignee,
            assigner,
            image: _,
            category,
            deleted,
        }: &MapDetails,
    ) -> Result<(), DbError> {
        let insert_token_query = if *deleted {
            sqlx::query("DELETE FROM maps WHERE id = ?").bind(id)
        } else {
            sqlx::query("UPDATE maps SET name = ?, file_name = ?, assignee = ?, assigner = ?, category = ? WHERE id = ?")
                .bind(name)
                .bind(image_name)
                .bind(assignee)
                .bind(assigner)
                .bind(category)
                .bind(id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- USERS -------------

    pub async fn get_user(&self, id: u32) -> Result<UserPublicDetails, DbError> {
        let query = sqlx::query("SELECT * FROM users WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let user_details_result = user_row_to_details(&rows[0]);
                    match user_details_result {
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

    pub async fn add_user(&self, details: UserPublicDetails) -> Result<(), DbError> {
        let insert_token_query = sqlx::query("INSERT INTO users(id, name) VALUES (?, ?)")
            .bind(details.id)
            .bind(details.name);

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_user(&self, details: UserPublicDetails) -> Result<(), DbError> {
        let insert_token_query = if details.deleted {
            sqlx::query("DELETE FROM users WHERE id = ?").bind(details.id)
        } else {
            sqlx::query("UPDATE users SET name = ? WHERE id = ?")
                .bind(details.name)
                .bind(details.id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- STREETS -------------

    pub async fn get_streets(&self, map_id: u32) -> Result<Vec<StreetDetails>, DbError> {
        let query = sqlx::query("SELECT * FROM streets WHERE map_id = ?").bind(&map_id);

        let rows_result = query.fetch_all(&self.data).await;

        let mut streets = vec![];

        match rows_result {
            Ok(rows) => {
                for row in rows {
                    let map_details_result = street_row_to_details(&row);
                    match map_details_result {
                        Ok(details) => streets.push(details),
                        Err(error) => return Err(DbError::InvalidRow(error)),
                    }
                }
            }
            Err(error) => return Err(DbError::QueryFailure(error)),
        }

        Ok(streets)
    }

    pub async fn get_street(&self, id: u32) -> Result<StreetDetails, DbError> {
        let query = sqlx::query("SELECT * FROM streets WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let map_details_result = street_row_to_details(&rows[0]);
                    match map_details_result {
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

    pub async fn add_street(&self, details: &StreetDetails) -> Result<(), DbError> {
        let insert_token_query =
            sqlx::query("INSERT INTO streets(id, map_id, name) VALUES (?, ?, ?)")
                .bind(details.id)
                .bind(details.map_id)
                .bind(details.name.clone());

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_street(&self, details: &StreetDetails) -> Result<(), DbError> {
        let insert_token_query = if details.deleted {
            sqlx::query("DELETE FROM streets WHERE id = ?").bind(details.id)
        } else {
            sqlx::query("UPDATE streets SET map_id = ?, name = ? WHERE id = ?")
                .bind(details.map_id)
                .bind(details.name.clone())
                .bind(details.id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- ADDRESSES -------------

    pub async fn get_addresses(
        &self,
        streets: &Vec<StreetDetails>,
    ) -> Result<Vec<AddressDetails>, DbError> {
        let mut addresses = vec![];
        for street in streets {
            let street_id = street.id;
            let query = sqlx::query("SELECT * FROM addresses WHERE street_id = ?").bind(street_id);

            let rows_result = query.fetch_all(&self.data).await;

            match rows_result {
                Ok(rows) => {
                    for row in rows {
                        let address_details_result = address_row_to_details(&row);
                        match address_details_result {
                            Ok(details) => addresses.push(details),
                            Err(error) => return Err(DbError::AddressFailure(error)),
                        }
                    }
                }
                Err(error) => return Err(DbError::QueryFailure(error)),
            }
        }
        Ok(addresses)
    }

    pub async fn get_address(&self, id: u32) -> Result<AddressDetails, DbError> {
        let query = sqlx::query("SELECT * FROM addresses WHERE id = ?").bind(&id);

        let rows_result = query.fetch_all(&self.data).await;

        match rows_result {
            Ok(rows) => {
                if rows.len() == 1 {
                    let user_details_result = address_row_to_details(&rows[0]);
                    match user_details_result {
                        Ok(details) => return Ok(details),
                        Err(error) => return Err(DbError::AddressFailure(error)),
                    }
                    // return Ok(user_id);
                } else {
                    return Err(DbError::InvalidToken(rows.len() as u32));
                }
            }
            Err(error) => return Err(DbError::QueryFailure(error)),
        }
    }

    pub async fn add_address(&self, details: &AddressDetails) -> Result<(), DbError> {
        let tags_encoded = rmp_serde::encode::to_vec(&details.tags)?;

        let insert_token_query = sqlx::query(
            "INSERT INTO addresses(id, street_id, number, tags, visited) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(details.id)
        .bind(details.street_id)
        .bind(details.number.clone())
        .bind(tags_encoded)
        .bind(details.visited);

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_address(&self, details: &AddressDetails) -> Result<(), DbError> {
        let insert_token_query = if details.deleted {
            sqlx::query("DELETE FROM addresses WHERE id = ?").bind(details.id)
        } else {
            let encoded_tags: Vec<u8> = rmp_serde::encode::to_vec(&details.tags)?;
            sqlx::query("UPDATE addresses SET street_id = ?, number = ?, tags = ?, visited = ? WHERE id = ?")
                .bind(details.street_id)
                .bind(details.number.clone())
                .bind(encoded_tags)
                .bind(details.visited)
                .bind(details.id)
        };

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    pub async fn update_address_checked(&self, id: u32, checked: bool) -> Result<(), DbError> {
        let insert_token_query = sqlx::query("UPDATE addresses SET visited = ? WHERE id = ?")
            .bind(checked)
            .bind(id);

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }

    // ------------------- LOGGING -------------

    pub async fn add_log(&self, code: u32, message: &str, timestamp: u32) -> Result<(), DbError> {
        let insert_token_query =
            sqlx::query("INSERT INTO logs(code, message, timestamp) VALUES (?, ?, ?)")
                .bind(code)
                .bind(message)
                .bind(timestamp);

        let query_result = insert_token_query.execute(&self.data).await;

        if let Err(result) = query_result {
            return Err(DbError::QueryFailure(result));
        }
        Ok(())
    }
}

fn cong_row_to_details(row: &SqliteRow) -> Result<CongDetails, sqlx::Error> {
    let cong_id: u32 = row.try_get("id")?;
    let cong_name: String = row.try_get("name")?;
    let updated: u32 = row.try_get("updated")?;
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
    let id = row.try_get("id")?;
    let name: String = row.try_get("name")?;
    let cong: u32 = row.try_get("congregation")?;
    let elder: u32 = row.try_get("elder")?;
    let updated: u32 = row.try_get("updated")?;
    Ok(GroupDetails {
        id,
        name,
        cong,
        elder,
        updated,
        group_deleted: false,
        pair_deleted: false,
    })
}

fn user_row_to_details(row: &SqliteRow) -> Result<UserPublicDetails, sqlx::Error> {
    let id = row.try_get("id")?;
    let name: String = row.try_get("name")?;
    Ok(UserPublicDetails {
        id,
        name,
        deleted: false,
    })
}

fn map_row_to_details(row: &SqliteRow) -> Result<MapDetails, sqlx::Error> {
    let id = row.try_get("id")?;
    let name = row.try_get("name")?;
    let assignee: u32 = row.try_get("assignee")?;
    let assigner: u32 = row.try_get("assigner")?;
    let image_name: String = row.try_get("file_name")?;
    let category: u32 = row.try_get("category")?;
    Ok(MapDetails {
        id,
        name,
        image_name,
        assignee,
        assigner,
        image: None,
        category,
        deleted: false,
    })
}

fn map_row_to_display_details(row: &SqliteRow) -> Result<MapDisplayDetails, sqlx::Error> {
    let id: u32 = row.try_get("id")?;
    let display_name: String = row.try_get("name")?;
    let file_name: String = row.try_get("file_name")?;
    let category: u32 = row.try_get("category")?;
    Ok(MapDisplayDetails {
        id,
        display_name,
        file_name,
        category,
    })
}

fn street_row_to_details(row: &SqliteRow) -> Result<StreetDetails, sqlx::Error> {
    let id = row.try_get("id")?;
    let map_id: u32 = row.try_get("map_id")?;
    let name: String = row.try_get("name")?;
    Ok(StreetDetails {
        id,
        map_id,
        name,
        deleted: false,
    })
}

fn address_row_to_details(row: &SqliteRow) -> Result<AddressDetails, AddressError> {
    let id = row.try_get("id")?;
    let street_id: u32 = row.try_get("street_id")?;
    let number: String = row.try_get("number")?;
    let tags_encoded: Vec<u8> = row.try_get("tags")?;
    let tags: Vec<AddressTags> = rmp_serde::from_slice(&tags_encoded)?;
    let visited = row.try_get("visited")?;
    Ok(AddressDetails {
        id,
        street_id,
        number,
        tags,
        visited,
        deleted: false,
    })
}

fn token_row_to_details(row: &SqliteRow) -> Result<[u8; 32], sqlx::Error> {
    let token_encoded: Vec<u8> = row.try_get("token")?;
    let token = hex::decode(token_encoded);
    match token {
        Ok(token) => {
            let mut token_array = [0u8; 32];
            token_array.copy_from_slice(&token);
            Ok(token_array)
        }
        // TODO: Actual error type
        Err(_error) => return Err(sqlx::Error::PoolTimedOut),
    }
}
