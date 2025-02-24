#[cfg(feature = "ssr")]
mod db_impl {
    use rusqlite::{Connection, Error};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use leptos::logging;
    use std::collections::HashMap;
    use crate::models::item::Item;
    use leptos::logging::log;

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

        async fn get_or_create_property(&self, prop: &str) -> Result<i64, Error> {
            let conn = self.conn.lock().await;
            // Check existing
            let exists: Result<i64, _> = conn.query_row(
                "SELECT id FROM properties WHERE name = ?",
                &[prop],
                |row| row.get(0)
            );
            
            match exists {
                Ok(id) => Ok(id),
                Err(_) => {
                    conn.execute(
                        "INSERT INTO properties (name) VALUES (?)",
                        &[prop],
                    )?;
                    Ok(conn.last_insert_rowid())
                }
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
            let tx = conn.transaction().map_err(|e| {
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
            log!("[DB] Inserting item into items table");
            match tx.execute(
                "INSERT INTO items (id, url_id, name, description, wikidata_id) 
                VALUES (?, ?, ?, ?, ?)",
                rusqlite::params![
                    &item.id,
                    url_id,
                    &item.name,
                    &item.description,
                    &item.wikidata_id
                ],
            ) {
                Ok(_) => log!("[DB] Item inserted successfully"),
                Err(e) => {
                    log!("[DB] Item insert error: {:?}", e);
                    return Err(e.into());
                }
            }
            // Property handling with enhanced logging
            log!("[DB] Processing {} properties", item.custom_properties.len());
            for (prop, value) in &item.custom_properties {
                log!("[DB] Handling property: {}", prop);

                // Property Lookup/Creation
                let prop_id = match tx.query_row(
                    "SELECT id FROM properties WHERE name = ?",
                    [prop],
                    |row| row.get::<_, i64>(0)
                ) {
                    Ok(id) => {
                        log!("[DB] Existing property ID: {}", id);
                        id
                    },
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        log!("[DB] Creating new property: {}", prop);
                        tx.execute("INSERT INTO properties (name) VALUES (?)", [prop])?;
                        let id = tx.last_insert_rowid();
                        log!("[DB] New property ID: {}", id);
                        id
                    }
                    Err(e) => {
                        log!("[DB] Property lookup error: {:?}", e);
                        return Err(e.into());
                    }
                };
                // Property Value Insertion
                log!("[DB] Inserting property {} with value {}", prop, value);
                tx.execute(
                    "INSERT INTO item_properties (item_id, property_id, value)
                    VALUES (?, ?, ?)",
                    rusqlite::params![&item.id, prop_id, &value],
                )?;
            }
        
            tx.commit()?;
            log!("[DB] Transaction committed successfully");
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
            
            // Delete from junction table instead of JSON
            conn.execute(
                "DELETE FROM item_properties 
                 WHERE property_id IN (
                     SELECT id FROM properties WHERE name = ?
                 ) AND item_id IN (
                     SELECT id FROM items WHERE url_id = ?
                 )",
                 rusqlite::params![
                     property,  // &str
                     url_id     // i64
                 ],
            )?;
            
            logging::log!("Property deleted from the database for URL: {}", url);
            Ok(())
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