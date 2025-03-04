#[cfg(feature = "ssr")]
mod db_impl {
    use rusqlite::{Connection, Error};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use leptos::logging;
    use std::collections::{HashMap, HashSet};
    use crate::models::item::Item;
    use leptos::logging::log;

    #[cfg(test)]
    mod tests {
        use super::*;
        use tokio::runtime::Runtime;
        use uuid::Uuid;

        // Helper function to create test database
        async fn create_test_db() -> Database {
            let db = Database::new(":memory:").unwrap();
            db.create_schema().await.unwrap();
            db
        }

        // Test database schema creation
        #[tokio::test]
        async fn test_schema_creation() {
            let db = create_test_db().await;
            // Verify tables exist
            let conn = db.conn.lock().await;
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
            let tables: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap();

            assert!(tables.contains(&"urls".to_string()));
            assert!(tables.contains(&"items".to_string()));
            assert!(tables.contains(&"properties".to_string()));
            assert!(tables.contains(&"item_properties".to_string()));
        }

        // Item Lifecycle Tests
        #[tokio::test]
        async fn test_full_item_lifecycle() {
            let db = create_test_db().await;
            let test_url = "https://example.com";
            let test_item = Item {
                id: Uuid::new_v4().to_string(),
                name: "Test Item".into(),
                description: "Test Description".into(),
                wikidata_id: Some("Q123".into()),
                custom_properties: vec![
                    ("price".into(), "100".into()),
                    ("color".into(), "red".into())
                ].into_iter().collect(),
            };
        
            // Test insertion
            db.insert_item_by_url(test_url, &test_item).await.unwrap();

            // Test retrieval
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items.len(), 1);
            let stored_item = &items[0];
            assert_eq!(stored_item.name, test_item.name);
            assert_eq!(stored_item.custom_properties.len(), 2);
        
            // Test update
            let mut updated_item = test_item.clone();
            updated_item.name = "Updated Name".into();
            db.insert_item_by_url(test_url, &updated_item).await.unwrap();

            // Verify update
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items[0].name, "Updated Name");
        
            // Test deletion
            db.delete_item_by_url(test_url, &test_item.id).await.unwrap();
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert!(items.is_empty());
        }

        //property management tests
        #[tokio::test]
        async fn test_property_operations() {
            let db = create_test_db().await;
            let test_url = "https://props.com";
            let test_item = Item {
                id: Uuid::new_v4().to_string(),
                name: "Test Item".into(),
                description: "Test Description".into(),
                wikidata_id: Some("Q123".into()),
                custom_properties: vec![
                    ("price".into(), "100".into()),
                    ("color".into(), "red".into())
                ].into_iter().collect(),
            };        
            // Test property creation
            db.insert_item_by_url(test_url, &test_item).await.unwrap();

            // Verify properties stored
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items[0].custom_properties.len(), 2);
        
            // Test property deletion
            db.delete_property_by_url(test_url, "price").await.unwrap();
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items[0].custom_properties.len(), 1);
            assert!(!items[0].custom_properties.contains_key("price"));
        }


    }

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
                    name TEXT NOT NULL UNIQUE, 
                    global_usage_count INTEGER DEFAULT 0
                );"
            ).map_err(|e| {
                eprintln!("Failed creating properties table: {}", e);
                e
            })?;

            // 2. URLs table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS urls (
                    id INTEGER PRIMARY KEY,
                    url TEXT NOT NULL UNIQUE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );",
            ).map_err(|e| {
                eprintln!("Failed creating urls table: {}", e);
                e
            })?;

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
            ).map_err(|e| {
                eprintln!("Failed creating items table: {}", e);
                e
            })?;

            // 4. Table for selected properties
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS selected_properties (
                    url_id INTEGER NOT NULL,
                    property_id INTEGER NOT NULL,
                    PRIMARY KEY (url_id, property_id),
                    FOREIGN KEY (url_id) REFERENCES urls(id) ON DELETE CASCADE,
                    FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE
                );"
            ).map_err(|e| {
                eprintln!("Failed creating properties table: {}", e);
                e
            })?;
            
            // 5. Junction table for custom properties
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS item_properties (
                    item_id TEXT NOT NULL,
                    property_id INTEGER NOT NULL,
                    value TEXT NOT NULL,
                    PRIMARY KEY (item_id, property_id),
                    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
                    FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE
                );"
            ).map_err(|e| {
                eprintln!("Failed creating item_properties table: {}", e);
                e
            })?;
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
        pub async fn get_items_by_url(&self, url: &str) -> Result<Vec<Item>, Error> {
            let conn = self.conn.lock().await;
            let url_id: Option<i64> = match conn.query_row(
                "SELECT id FROM urls WHERE url = ?",
                &[url],
                |row| row.get(0)
            ) {
                Ok(id) => Some(id),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(e),
            };
        
            let url_id = match url_id {
                Some(id) => id,
                None => return Ok(Vec::new()), // Return empty list if URL not found
            };

            log!("Fetching items for URL '{}' (ID: {})", url, url_id);

            
            let mut stmt = conn.prepare(
                "SELECT i.id, i.name, i.description, i.wikidata_id, 
                        p.name AS prop_name, ip.value
                 FROM items i
                 LEFT JOIN item_properties ip ON i.id = ip.item_id
                 LEFT JOIN properties p ON ip.property_id = p.id
                 WHERE i.url_id = ?"
            )?;
            let mut items: HashMap<String, Item> = HashMap::new();
            
            let rows = stmt.query_map([url_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,  // id
                    row.get::<_, String>(1)?,  // name
                    row.get::<_, String>(2)?,  // description
                    row.get::<_, Option<String>>(3)?,  // wikidata_id
                    row.get::<_, Option<String>>(4)?,  // prop_name
                    row.get::<_, Option<String>>(5)?,  // value
                ))
            })?;
        
            for row in rows {
                let (id, name, desc, wd_id, prop, val) = row?;
                let item = items.entry(id.clone()).or_insert(Item {
                    id,
                    name,
                    description: desc,
                    wikidata_id: wd_id,
                    custom_properties: HashMap::new(),
                });
                
                if let (Some(p), Some(v)) = (prop, val) {
                    item.custom_properties.insert(p, v);
                }
            }
        
            Ok(items.into_values().collect())
        }

        async fn get_or_create_property(
            &self, 
            tx: &mut rusqlite::Transaction<'_>, 
            prop: &str
        ) -> Result<i64, Error> {
            match tx.query_row(
                "SELECT id FROM properties WHERE name = ?",
                [prop],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(id) => Ok(id),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    tx.execute("INSERT INTO properties (name) VALUES (?)", [prop])?;
                    Ok(tx.last_insert_rowid())
                }
                Err(e) => Err(e.into()),
            }
        }

        // Insert a new item into the database for a specific URL
        pub async fn insert_item_by_url(&self, url: &str, item: &Item) -> Result<(), Error> {
            log!("[DB] Starting insert for URL: {}, Item: {}", url, item.id);
            
            // 1. Check database lock acquisition
            let lock_start = std::time::Instant::now();
            let mut conn = self.conn.lock().await;
            log!("[DB] Lock acquired in {:?}", lock_start.elapsed());
        
            // 2. Transaction handling
            log!("[DB] Starting transaction");
            let mut tx = conn.transaction().map_err(|e| {
                log!("[DB] Transaction start failed: {:?}", e);
                e
            })?;
        
            // 3. URL handling
            log!("[DB] Checking URL existence: {}", url);
            let url_id = match tx.query_row(
                "SELECT id FROM urls WHERE url = ?",
                [url],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(id) => {
                    log!("[DB] Found existing URL ID: {}", id);
                    id
                },
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    log!("[DB] Inserting new URL");
                    tx.execute("INSERT INTO urls (url) VALUES (?)", [url])?;
                    let id = tx.last_insert_rowid();
                    log!("[DB] Created URL ID: {}", id);
                    id
                }
                Err(e) => return Err(e.into()),
            };
        
            // 4. Item insertion
            log!("[DB] Upserting item");
            tx.execute(
                "INSERT INTO items (id, url_id, name, description, wikidata_id)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    url_id = excluded.url_id,
                    name = excluded.name,
                    description = excluded.description,
                    wikidata_id = excluded.wikidata_id",
                rusqlite::params![
                    &item.id,
                    url_id,
                    &item.name,
                    &item.description,
                    &item.wikidata_id
                ],
            )?;
            log!("[DB] Item upserted successfully");
            // Property handling with enhanced logging
            log!("[DB] Synchronizing properties for item {}", item.id);
            let existing_props = {
                // Prepare statement and collect existing properties
                let mut stmt = tx.prepare(
                    "SELECT p.name, ip.value 
                    FROM item_properties ip
                    JOIN properties p ON ip.property_id = p.id
                    WHERE ip.item_id = ?"
                )?;

                let mapped_rows = stmt.query_map([&item.id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;

                mapped_rows.collect::<Result<HashMap<String, String>, _>>()?
            };
        
            for (prop, value) in &item.custom_properties {
                // Update existing or insert new
                let prop_id = self.get_or_create_property(&mut tx, prop).await?;
                if let Some(existing_value) = existing_props.get(prop) {
                    if existing_value != value {
                        log!("[DB] Updating property {} from '{}' to '{}'", prop, existing_value, value);
                        tx.execute(
                            "UPDATE item_properties 
                            SET value = ? 
                            WHERE item_id = ? 
                            AND property_id = (SELECT id FROM properties WHERE name = ?)",
                            rusqlite::params![value, &item.id, prop],
                        )?;
                    }
                } else {
                    log!("[DB] Adding new property {}", prop);
                    tx.execute(
                        "INSERT INTO item_properties (item_id, property_id, value)
                        VALUES (?, ?, ?)",
                        rusqlite::params![&item.id, prop_id, value],
                    )?;
                }
            }

            // Remove deleted properties
            let current_props: HashSet<&str> = item.custom_properties.keys().map(|s| s.as_str()).collect();
            for (existing_prop, _) in existing_props {
                if !current_props.contains(existing_prop.as_str()) {
                    log!("[DB] Removing deleted property {}", existing_prop);
                    tx.execute(
                        "DELETE FROM item_properties 
                        WHERE item_id = ? 
                        AND property_id = (SELECT id FROM properties WHERE name = ?)",
                        rusqlite::params![&item.id, existing_prop],
                    )?;
                }
            }
            tx.commit()?;
            log!("[DB] Transaction committed successfully");
            Ok(())
        }

        // Delete an item from the database for a specific URL
        pub async fn delete_item_by_url(&self, url: &str, item_id: &str) -> Result<(), Error> {
            let mut conn = self.conn.lock().await;
            let tx = conn.transaction()?;

            // Get URL ID
            let url_id: i64 = tx.query_row(
                "SELECT id FROM urls WHERE url = ?",
                [url],
                |row| row.get(0)
            )?;
        
            // Delete item and properties
            tx.execute(
                "DELETE FROM items WHERE id = ? AND url_id = ?",
                [item_id, &url_id.to_string()],
            )?;
        
            tx.commit()?;
            Ok(())
        }

        // Delete a property from the database for a specific URL
        pub async fn delete_property_by_url(&self, url: &str, property: &str) -> Result<(), Error> {
            let mut conn = self.conn.lock().await;
            let tx = conn.transaction()?;
            
            // Get URL ID
            let url_id: i64 = tx.query_row(
                "SELECT id FROM urls WHERE url = ?",
                [url],
                |row| row.get(0)
            )?;
        
            // Delete property from all items in this URL
            tx.execute(
                "DELETE FROM item_properties 
                WHERE property_id IN (
                    SELECT id FROM properties WHERE name = ?
                )
                AND item_id IN (
                    SELECT id FROM items WHERE url_id = ?
                )",
                [property, &url_id.to_string()],
            )?;
        
            tx.commit()?;
            Ok(())
        }

        pub async fn add_selected_property(&self, url: &str, property: &str) -> Result<(), Error> {
            let mut conn = self.conn.lock().await;
            let tx = conn.transaction()?;
            
            // Get URL ID
            let url_id = tx.query_row(
                "SELECT id FROM urls WHERE url = ?",
                [url],
                |row| row.get::<_, i64>(0)
            )?;
        
            // Get/Create property
            let prop_id = match tx.query_row(
                "SELECT id FROM properties WHERE name = ?",
                [property],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(id) => id,
                Err(_) => {
                    tx.execute("INSERT INTO properties (name) VALUES (?)", [property])?;
                    tx.last_insert_rowid()
                }
            };
        
            // Insert into selected_properties
            tx.execute(
                "INSERT OR IGNORE INTO selected_properties (url_id, property_id) VALUES (?, ?)",
                [url_id, prop_id]
            )?;
        
            tx.commit()?;
            Ok(())
        }

        pub async fn get_selected_properties(&self, url: &str) -> Result<Vec<String>, Error> {
            let conn = self.conn.lock().await;
            let mut stmt = conn.prepare(
                "SELECT p.name 
                 FROM selected_properties sp
                 JOIN properties p ON sp.property_id = p.id
                 JOIN urls u ON sp.url_id = u.id
                 WHERE u.url = ?"
            )?;
            
            let properties = stmt.query_map([url], |row| row.get(0))?;
            properties.collect()
        }
        
        // function to log database state
        pub async fn debug_dump(&self) -> Result<(), Error> {
            let conn = self.conn.lock().await;
            log!("[DATABASE DEBUG] URLs:");
            let mut stmt = conn.prepare("SELECT id, url FROM urls")?;
            let urls = stmt.query_map([], |row| {
                Ok(format!("ID: {}, URL: {}", row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            for url in urls {
                log!("[DATABASE DEBUG] {}", url?);
            }
        
            log!("[DATABASE DEBUG] Items:");
            let mut stmt = conn.prepare("SELECT id, name FROM items")?;
            let items = stmt.query_map([], |row| {
                Ok(format!("ID: {}, Name: '{}'", row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for item in items {
                log!("[DATABASE DEBUG] {}", item?);
            }
        
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
    }
}

#[cfg(feature = "ssr")]
pub use db_impl::{Database, DbItem};