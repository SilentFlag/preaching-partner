use crate::database::MyDatabase;
use crate::datatypes::{
    CategoryDetails, CongDetails, DbError, GroupDetails, ServerPayload, SyncInformation,
    UserPublicDetails,
};
use std::fs::File;
use std::io::Write;

/// function to call for a full sync with the server
pub async fn sync_with_server(db: MyDatabase, sync_info: SyncInformation) -> Result<(), DbError> {
    println!("syncing with server");
    let congregations = sync_info.congregations;
    let categories = sync_info.categories;
    let service_groups = sync_info.service_groups;
    let users = sync_info.users;

    // Sync Congregations
    let cong_sync_result = sync_congregations(congregations, db.clone()).await;
    if let Err(cong_error) = cong_sync_result {
        println!("Syncing congregations failed: {}", cong_error);
    }

    // Sync Categories
    let category_sync_result = sync_categories(categories, db.clone()).await;
    if let Err(category_error) = category_sync_result {
        println!("Syncing categories failed: {}", category_error);
    }

    // TODO: Sync users here, before groups
    let user_sync_result = sync_users(users, db.clone()).await;
    if let Err(user_error) = user_sync_result {
        println!("Syncing users failed: {}", user_error);
    }

    // Sync Service Groups
    let service_group_result = sync_service_groups(service_groups, db.clone()).await;
    if let Err(service_error) = service_group_result {
        println!("Syncing groups failed: {}", service_error);
    }

    Ok(())
}

async fn sync_congregations(
    congregations: Vec<CongDetails>,
    db: MyDatabase,
) -> Result<(), DbError> {
    for cong in congregations {
        let cong_result = db.get_congregation(cong.cong_id).await;
        match cong_result {
            Ok(_) => {
                let _result = db.update_congregation(cong).await;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    let add_cong_result = db.add_congregation(cong).await;
                    if let Err(error) = add_cong_result {
                        return Err(error);
                    }
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}

async fn sync_categories(categories: Vec<CategoryDetails>, db: MyDatabase) -> Result<(), DbError> {
    for category in categories {
        let category_result = db.get_category(category.id).await;
        match category_result {
            Ok(_) => {
                // TODO: Failure to update a category error
                let _result = db.update_category(category).await;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    let add_category_result = db.add_category(category).await;
                    if let Err(error) = add_category_result {
                        return Err(error);
                    }
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}

async fn sync_service_groups(groups: Vec<GroupDetails>, db: MyDatabase) -> Result<(), DbError> {
    for group in groups {
        let group_result = db.get_service_group(group.id).await;
        match group_result {
            Ok(_) => {
                let _result = db.update_service_group(group).await;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    let add_group_result = db.add_service_group(group).await;
                    if let Err(error) = add_group_result {
                        return Err(error);
                    }
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}

// TODO: Write this, it is currently copied and pasted sync_service_groups()
async fn sync_users(users: Vec<UserPublicDetails>, db: MyDatabase) -> Result<(), DbError> {
    for user in users {
        let group_result = db.get_user(user.id).await;
        match group_result {
            Ok(_) => {
                let _result = db.update_user(user).await;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    let add_group_result = db.add_user(user).await;
                    if let Err(error) = add_group_result {
                        return Err(error);
                    }
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}

// async fn _save_image(
//     image_payload: ServerPayload,
//     timestamp: Vec<u8>,
//     db: &sqlx::Pool<sqlx::Sqlite>,
// ) -> Result<(), ()> {
//     match image_payload {
//         ServerPayload::MapImage {
//             image_name,
//             image,
//             assignee,
//             assigner,
//             category,
//         } => {
//             let new_image_file = File::create(format!("../maps/{}", image_name.as_str()));
//             if let Ok(mut image_file) = new_image_file {
//                 let attempt_to_write = image_file.write(&image);
//                 if let Ok(_) = attempt_to_write {
//                     println!("Successfully saved the image");
//                 }
//             } else {
//                 println!("Failed to create image file");
//             }

//             // TODO: Save to database

//             let insert_image_query = sqlx::query(
//         "INSERT INTO maps(assignee, assigner, category, file_name, updated) VALUES (?,?,?,?,?,?)",
//     )
//     .bind(assignee)
//     .bind(assigner)
//     .bind(category)
//     .bind(hex::encode(image_name))
//     .bind(hex::encode(timestamp));

//             let query_result = insert_image_query.execute(db).await;

//             if let Err(result) = query_result {
//                 // TODO: handle this error
//                 println!(
//                     "Something went wrong inserting refresh token into database, error: {:?}",
//                     result
//                 )
//             }
//             return Ok(());
//         }
//         _ => {
//             return Err(());
//         }
//     }
// }
