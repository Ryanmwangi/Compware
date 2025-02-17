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
            logging::log!("Database connection established at: {}", db_path); 
            Ok(Database {
                conn: Arc::new(Mutex::new(conn)),
            })
        }

        // Create the database schema
        pub async fn create_schema(&self) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            
            // 1. Properties table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS properties (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,  // Property name
                    global_usage_count INTEGER DEFAULT 0  // Track usage across ALL URLs
                );"
            )?;

            // 2. URLs table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS urls (
                    id INTEGER PRIMARY KEY,
                    url TEXT NOT NULL UNIQUE,  // Enforce unique URLs
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );",
            )?;
            logging::log!("URLs table created or verified");

            // 3. Items table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS items (
                    id TEXT PRIMARY KEY,
                    url_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    wikidata_id TEXT,
                    FOREIGN KEY (url_id) REFERENCES urls(id) ON DELETE CASCADE
                );",
            )?;
            logging::log!("Items table updated with foreign key to URLs table");
            
            // 4. Junction table for custom properties
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS item_properties (
                    item_id TEXT NOT NULL,
                    property_id INTEGER NOT NULL,
                    value TEXT NOT NULL,
                    PRIMARY KEY (item_id, property_id),
                    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
                    FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE
                );"
            )?;
            Ok(())
        }

        // Insert a new URL into the database
        pub async fn insert_url(&self, url: &str) -> Result<i64, Error> {
            let conn = self.conn.lock().await;
            let mut stmt = conn.prepare("INSERT INTO urls (url) VALUES (?)")?;
            let url_id = stmt.insert(&[url])?;
            logging::log!("URL inserted: {}", url);
            Ok(url_id)
        }

        // Insert a new item into the database
        pub async fn insert_item(&self, url_id: i64, item: &DbItem) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            let wikidata_id = item.wikidata_id.as_ref().map(|s| s.as_str()).unwrap_or("");
            conn.execute(
            "INSERT INTO items (id, name, description, wikidata_id, custom_properties, url_id)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 description = excluded.description,
                 wikidata_id = excluded.wikidata_id,
                 custom_properties = excluded.custom_properties;",
            &[
                &item.id,
                &item.name,
                &item.description,
                &wikidata_id.to_string(),
                &item.custom_properties,
                &url_id.to_string(),
            ],
        )?;
            logging::log!("Item inserted: {}", item.id);
            Ok(())
        }

        pub async fn delete_item(&self, item_id: &str) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            conn.execute("DELETE FROM items WHERE id = ?", &[item_id])?;
            logging::log!("Item deleted: {}", item_id);
            Ok(())
        }

        pub async fn delete_property(&self, property: &str) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            let query = format!("UPDATE items SET custom_properties = json_remove(custom_properties, '$.{}')", property);
            conn.execute(&query, []).map_err(|e| Error::from(e))?;
            logging::log!("Property deleted: {}", property);
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

        // Retrieve all items from the database for a specific URL
        pub async fn get_items_by_url(&self, url: &str) -> Result<Vec<DbItem>, Error> {
            let conn = self.conn.lock().await;
            let url_id: i64  = conn.query_row("SELECT id FROM urls WHERE url = ?", &[url], |row| row.get(0))?;
            let mut stmt = conn.prepare("SELECT * FROM items WHERE url_id = ?")?;
            let items = stmt.query_map(&[&url_id], |row| {
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
            logging::log!("Fetched {} items from the database for URL: {}", result.len(), url);
            Ok(result)
        }

        // Insert a new item into the database for a specific URL
        pub async fn insert_item_by_url(&self, url: &str, item: &DbItem) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            let url_id: i64 = conn.query_row("SELECT id FROM urls WHERE url = ?", &[url], |row| row.get(0))?;
            if url_id == 0 {
                let url_id = self.insert_url(url).await?;
                self.insert_item(url_id, item).await?;
            } else {
                self.insert_item(url_id, item).await?;
            }
            logging::log!("Item inserted into the database for URL: {}", url);
            Ok(())
        }

        // Delete an item from the database for a specific URL
        pub async fn delete_item_by_url(&self, url: &str, item_id: &str) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            let url_id: i64 = conn.query_row("SELECT id FROM urls WHERE url = ?", &[url], |row| row.get(0))?;
            conn.execute("DELETE FROM items WHERE id = ? AND url_id = ?", &[item_id, &url_id.to_string()])?;
            logging::log!("Item deleted from the database for URL: {}", url);
            Ok(())
        }

        // Delete a property from the database for a specific URL
        pub async fn delete_property_by_url(&self, url: &str, property: &str) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            let url_id: i64 = conn.query_row("SELECT id FROM urls WHERE url = ?", &[url], |row| row.get(0))?;
            let query = format!("UPDATE items SET custom_properties = json_remove(custom_properties, '$.{}') WHERE url_id = ?", property);
            conn.execute(&query, &[&url_id.to_string()])?;
            logging::log!("Property deleted from the database for URL: {}", url);
            Ok(())
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