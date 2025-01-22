#[cfg(feature = "ssr")]
use leptos::*;
#[cfg(feature = "ssr")]
use leptos::logging::log;
#[cfg(feature = "ssr")]
use crate::db::{Database, DbItem};

#[cfg(feature = "ssr")]
#[server(UpdateItem, "/api")]
pub async fn update_item_db(client_db_item: ClientDbItem) -> Result<(), ServerFnError> {

    let db_item = DbItem {
        id: client_db_item.id,
        name: client_db_item.name,
        description: client_db_item.description,
        wikidata_id: client_db_item.wikidata_id,
        custom_properties: client_db_item.custom_properties,
    };

    // Log the start of the function
    log!("Starting update_item function for item: {:?}", db_item);

    // Open the database
    let db = match Database::new("items.db") {
        Ok(db) => {
            log!("Database opened successfully");
            db
        }
        Err(e) => {
            log!("Failed to open database: {}", e);
            return Err(ServerFnError::ServerError(e.to_string()));
        }
    };

    // Insert the item into the database
    match db.insert_item(&db_item) {
        Ok(_) => {
            log!("Item inserted successfully: {:?}", db_item);
            Ok(())
        }
        Err(e) => {
            log!("Failed to insert item into database: {}", e);
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}