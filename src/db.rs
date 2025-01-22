#[cfg(feature = "ssr")]
mod db_impl {
    use rusqlite::{Connection, Error};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use leptos::logging;

    // Define a struct to represent a database connection
    #[derive(Debug)]
    pub struct Database {
        conn: Arc<Mutex<Connection>>,
    }

    impl Database {
        // Create a new database connection
        pub fn new(db_path: &str) -> Result<Self, Error> {
            let conn = Connection::open(db_path)?;
            logging::log!("Database connection established at: {}", db_path); // Log with Leptos
            Ok(Database {
                conn: Arc::new(Mutex::new(conn)),
            })
        }

        // Create the database schema
        pub async fn create_schema(&self) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS items (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    wikidata_id TEXT,
                    custom_properties TEXT
                );",
            )?;
            logging::log!("Database schema created or verified"); // Log with Leptos
            Ok(())
        }

        // Insert a new item into the database
        pub async fn insert_item(&self, item: &DbItem) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            let wikidata_id = item.wikidata_id.as_ref().map(|s| s.as_str()).unwrap_or("");
            conn.execute(
                "INSERT INTO items (id, name, description, wikidata_id, custom_properties) VALUES (?, ?, ?, ?, ?);",
                &[
                    &item.id,
                    &item.name,
                    &item.description,
                    &wikidata_id.to_string(),
                    &item.custom_properties,
                ],
            )?;
            logging::log!("Item inserted: {}", item.id); // Log with Leptos
            Ok(())
        }

        // Retrieve all items from the database
        pub async fn get_items(&self) -> Result<Vec<DbItem>, Error> {
            let conn = self.conn.lock().await;
            let mut stmt = conn.prepare("SELECT * FROM items;")?;
            let items = stmt.query_map([], |row| {
                Ok(DbItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    wikidata_id: row.get(3)?,
                    custom_properties: row.get(4)?,
                })
            })?;
            let mut result = Vec::new();
            for item in items {
                result.push(item?);
            }
            logging::log!("Fetched {} items from the database", result.len()); // Log with Leptos
            Ok(result)
        }
    }

    // Define a struct to represent an item in the database
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct DbItem {
        pub id: String,
        pub name: String,
        pub description: String,
        pub wikidata_id: Option<String>,
        pub custom_properties: String,
    }

    // Implement conversion from DbItem to a JSON-friendly format
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ItemResponse {
        pub id: String,
        pub name: String,
        pub description: String,
        pub wikidata_id: Option<String>,
        pub custom_properties: String,
    }

    impl From<DbItem> for ItemResponse {
        fn from(item: DbItem) -> Self {
            ItemResponse {
                id: item.id,
                name: item.name,
                description: item.description,
                wikidata_id: item.wikidata_id,
                custom_properties: item.custom_properties,
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub use db_impl::{Database, DbItem, ItemResponse};