use crate::database::MyDatabase;
use crate::datatypes::{
    AddressDetails, CategoryDetails, CongDetails, DbError, GroupDetails, MapDetails, StreetDetails,
    SyncInformation, UserPublicDetails,
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
    let maps = sync_info.maps;
    let streets = sync_info.streets;
    let addresses = sync_info.addresses;

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

    // Sync users
    let user_sync_result = sync_users(users, db.clone()).await;
    if let Err(user_error) = user_sync_result {
        println!("Syncing users failed: {}", user_error);
    }

    // Sync Service Groups
    let service_group_result = sync_service_groups(service_groups, db.clone()).await;
    if let Err(service_error) = service_group_result {
        println!("Syncing groups failed: {}", service_error);
    }

    // TODO: Sync maps here
    let maps_result = sync_maps(maps, db.clone()).await;
    if let Err(maps_error) = maps_result {
        println!("Syncing maps failed: {}", maps_error);
    }

    // TODO: Sync streets
    let streets_result = sync_streets(streets, db.clone()).await;
    if let Err(street_error) = streets_result {
        println!("Syncing streets failed: {}", street_error);
    }

    // TODO: Sync addresses? maybe not, load when click on app
    let addresses_result = sync_addresses(addresses, db.clone()).await;
    if let Err(addresses_error) = addresses_result {
        println!("Syncing addresses failed: {}", addresses_error);
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

async fn sync_users(users: Vec<UserPublicDetails>, db: MyDatabase) -> Result<(), DbError> {
    for user in users {
        let user_result = db.get_user(user.id).await;
        match user_result {
            Ok(_) => {
                let _result = db.update_user(user).await?;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    let _add_group_result = db.add_user(user).await?;
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}

async fn sync_maps(maps: Vec<MapDetails>, db: MyDatabase) -> Result<(), DbError> {
    for map in maps {
        let map_result = db.get_map(map.id).await;
        match map_result {
            Ok(_) => {
                let _result = db.update_map(&map).await?;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    // TODO: Check if map not deleted
                    let _add_map_result = db.add_map(&map).await?;
                }
                _ => {
                    return Err(err);
                }
            },
        }

        match map.image {
            Some(image) => {
                let _save_image_result = save_image(image, &map.image_name).await;
            }
            None => {}
        }
    }
    Ok(())
}

async fn save_image(image_payload: Vec<u8>, image_name: &str) -> Result<(), ()> {
    let new_image_file = File::create(format!("../maps/{}", image_name));
    if let Ok(mut image_file) = new_image_file {
        let attempt_to_write = image_file.write(&image_payload);
        if let Ok(_) = attempt_to_write {
            println!("Successfully saved the image");
            return Ok(());
        } else {
            return Err(());
        }
    } else {
        println!("Failed to create image file");
        return Err(());
    }
}

async fn sync_streets(streets: Vec<StreetDetails>, db: MyDatabase) -> Result<(), DbError> {
    for street in streets {
        let street_result = db.get_street(street.id).await;
        match street_result {
            Ok(_) => {
                let _result = db.update_street(&street).await?;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    // TODO: Check if street not deleted
                    let _add_street_result = db.add_street(&street).await?;
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}

async fn sync_addresses(addresses: Vec<AddressDetails>, db: MyDatabase) -> Result<(), DbError> {
    for address in addresses {
        let address_result = db.get_address(address.id).await;
        match address_result {
            Ok(_) => {
                let _result = db.update_address(&address).await?;
            }
            Err(err) => match err {
                DbError::InvalidToken(_id) => {
                    // TODO: Check if address not deleted
                    let _add_street_result = db.add_address(&address).await?;
                }
                _ => {
                    return Err(err);
                }
            },
        }
    }
    Ok(())
}
