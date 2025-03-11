#[cfg(feature = "ssr")]
mod db_impl {
    use crate::models::item::Item;
    use leptos::logging;
    use leptos::logging::log;
    use rusqlite::{Connection, Error};
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[cfg(test)]
    mod tests {
        use super::*;
        use tokio::runtime::Runtime;
        use uuid::Uuid;

        // Helper function to create test database
        async fn create_test_db() -> Database {
            log!("[TEST] Creating in-memory test database");
            let db = Database::new(":memory:").unwrap();
            db.create_schema().await.unwrap();
            log!("[TEST] Database schema created");
            db
        }

        // Test database schema creation
        #[tokio::test]
        async fn test_schema_creation() {
            log!("[TEST] Starting test_schema_creation");
            let db = create_test_db().await;

            // Verify tables exist
            let conn = db.conn.lock().await;
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table'")
                .unwrap();
            let tables: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();

            assert!(tables.contains(&"urls".to_string()));
            assert!(tables.contains(&"items".to_string()));
            assert!(tables.contains(&"properties".to_string()));
            assert!(tables.contains(&"item_properties".to_string()));
            assert!(tables.contains(&"selected_properties".to_string()));
        }

        // Item Lifecycle Tests
        #[tokio::test]
        async fn test_full_item_lifecycle() {
            log!("[TEST] Starting test_full_item_lifecycle");
            let db = create_test_db().await;
            let test_url = "https://example.com";
            let test_item = Item {
                id: Uuid::new_v4().to_string(),
                name: "Test Item".into(),
                description: "Test Description".into(),
                wikidata_id: Some("Q123".into()),
                custom_properties: vec![
                    ("price".into(), "100".into()),
                    ("color".into(), "red".into()),
                ]
                .into_iter()
                .collect(),
            };

            // Test insertion
            log!("[TEST] Testing item insertion");
            db.insert_item_by_url(test_url, &test_item).await.unwrap();
            log!("[TEST] Item insertion - PASSED");

            // Test retrieval
            log!("[TEST] Testing item retrieval");
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items.len(), 1);
            let stored_item = &items[0];
            assert_eq!(stored_item.name, test_item.name);
            assert_eq!(stored_item.custom_properties.len(), 2);
            log!("[TEST] Item retrieval and validation - PASSED");

            // Test update
            log!("[TEST] Testing item update");
            let mut updated_item = test_item.clone();
            updated_item.name = "Updated Name".into();
            db.insert_item_by_url(test_url, &updated_item)
                .await
                .unwrap();

            // Verify update
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items[0].name, "Updated Name");
            log!("[TEST] Item update - PASSED");

            // Test deletion
            log!("[TEST] Testing item deletion");
            db.delete_item_by_url(test_url, &test_item.id)
                .await
                .unwrap();
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert!(items.is_empty());
            log!("[TEST] Item deletion - PASSED");
            log!("[TEST] test_full_item_lifecycle completed successfully");
        }

        //URL Management Tests
        #[tokio::test]
        async fn test_url_management() {
            log!("[TEST] Starting test_url_management");
            let db = create_test_db().await;
            let test_url = "https://test.com";

            // Test URL creation
            log!("[TEST] Testing URL creation");
            let url_id = db.insert_url(test_url).await.unwrap();
            assert!(url_id > 0);
            log!("[TEST] URL creation - PASSED");

            // Test duplicate URL handling
            log!("[TEST] Testing duplicate URL handling");
            let duplicate_id = db.insert_url(test_url).await.unwrap();
            assert_eq!(url_id, duplicate_id);
            log!("[TEST] Duplicate URL handling - PASSED");

            // Test URL retrieval
            log!("[TEST] Testing URL retrieval");
            let conn = db.conn.lock().await;
            let stored_url: String = conn
                .query_row("SELECT url FROM urls WHERE id = ?", [url_id], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(stored_url, test_url);
            log!("[TEST] URL retrieval - PASSED");

            log!("[TEST] test_url_management completed successfully");
        }

        //property management tests
        #[tokio::test]
        async fn test_property_operations() {
            log!("[TEST] Starting test_property_operations");
            let db = create_test_db().await;
            let test_url = "https://props.com";
            let test_item = Item {
                id: Uuid::new_v4().to_string(),
                name: "Test Item".into(),
                description: "Test Description".into(),
                wikidata_id: Some("Q123".into()),
                custom_properties: vec![
                    ("price".into(), "100".into()),
                    ("color".into(), "red".into()),
                ]
                .into_iter()
                .collect(),
            };
            // Test property creation
            log!("[TEST] Testing property creation");
            db.insert_item_by_url(test_url, &test_item).await.unwrap();

            // Verify properties stored
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items[0].custom_properties.len(), 2);
            log!("[TEST] Property creation - PASSED");

            // Test property deletion
            log!("[TEST] Testing property deletion");
            db.delete_property_by_url(test_url, "price").await.unwrap();
            let items = db.get_items_by_url(test_url).await.unwrap();
            assert_eq!(items[0].custom_properties.len(), 1);
            assert!(!items[0].custom_properties.contains_key("price"));
            log!("[TEST] Property deletion - PASSED");

            log!("[TEST] test_property_operations completed successfully");
        }

        //selected properties test
        #[tokio::test]
        async fn test_selected_properties() {
            log!("[TEST] Starting test_selected_properties");
            let db = create_test_db().await;
            let test_url = "https://selected.com";

            // Add test properties
            log!("[TEST] Adding selected properties");
            db.add_selected_property(test_url, "price").await.unwrap();
            db.add_selected_property(test_url, "weight").await.unwrap();

            // Test retrieval
            log!("[TEST] Testing property retrieval");
            let props = db.get_selected_properties(test_url).await.unwrap();
            assert_eq!(props.len(), 2);
            assert!(props.contains(&"price".to_string()));
            assert!(props.contains(&"weight".to_string()));
            log!("[TEST] Property retrieval - PASSED");

            // Test duplicate prevention
            log!("[TEST] Testing duplicate prevention");
            db.add_selected_property(test_url, "price").await.unwrap();
            let props = db.get_selected_properties(test_url).await.unwrap();
            assert_eq!(props.len(), 2); // No duplicate added
            log!("[TEST] Duplicate prevention - PASSED");

            log!("[TEST] test_selected_properties completed successfully");
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
                );",
            )
            .map_err(|e| {
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
            )
            .map_err(|e| {
                eprintln!("Failed creating urls table: {}", e);
                e
            })?;

            // 3. Items table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS items (
                    id TEXT PRIMARY KEY,
                    url_id INTEGER NOT NULL,
                    wikidata_id TEXT,
                    item_order INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY (url_id) REFERENCES urls(id) ON DELETE CASCADE
                );
                INSERT OR IGNORE INTO properties (name) VALUES 
                ('name'), 
                ('description');",
            )
            .map_err(|e| {
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
                );",
            )
            .map_err(|e| {
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
                );",
            )
            .map_err(|e| {
                eprintln!("Failed creating item_properties table: {}", e);
                e
            })?;
            Ok(())
        }

        // Insert a new URL into the database
        pub async fn insert_url(&self, url: &str) -> Result<i64, Error> {
            let mut conn = self.conn.lock().await;
            let tx = conn.transaction()?;

            // Use INSERT OR IGNORE to handle duplicates
            tx.execute("INSERT OR IGNORE INTO urls (url) VALUES (?)", [url])?;

            // Get the URL ID whether it was inserted or already existed
            let url_id =
                tx.query_row("SELECT id FROM urls WHERE url = ?", [url], |row| row.get(0))?;

            tx.commit()?;
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
            let query = format!(
                "UPDATE items SET custom_properties = json_remove(custom_properties, '$.{}')",
                property
            );
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
            let url_id: Option<i64> =
                match conn.query_row("SELECT id FROM urls WHERE url = ?", &[url], |row| {
                    row.get(0)
                }) {
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
                "WITH ordered_items AS (
                    SELECT 
                        i.id,
                        i.wikidata_id,
                        i.item_order
                    FROM items i
                    WHERE i.url_id = ?
                    ORDER BY i.item_order ASC
                )
                SELECT
                    oi.id,
                    oi.wikidata_id,
                    name_ip.value AS name,
                    desc_ip.value AS description,
                    json_group_object(p.name, ip.value) as custom_properties
                FROM ordered_items oi
                LEFT JOIN item_properties ip 
                    ON oi.id = ip.item_id
                    AND ip.property_id NOT IN (
                        SELECT id FROM properties WHERE name IN ('name', 'description')
                    )
                LEFT JOIN properties p 
                    ON ip.property_id = p.id
                LEFT JOIN item_properties name_ip 
                    ON oi.id = name_ip.item_id
                    AND name_ip.property_id = (SELECT id FROM properties WHERE name = 'name')
                LEFT JOIN item_properties desc_ip 
                    ON oi.id = desc_ip.item_id
                    AND desc_ip.property_id = (SELECT id FROM properties WHERE name = 'description')
                GROUP BY oi.id
                ORDER BY oi.item_order ASC"
            )?;
        
            // Change from HashMap to Vec to preserve order
        
            let rows = stmt.query_map([url_id], |row| {
                  let custom_props_json: String = row.get(4)?;
                  let custom_properties: HashMap<String, String> = serde_json::from_str(&custom_props_json)
                      .unwrap_or_default();
    
                  Ok(Item {
                      id: row.get(0)?,
                      name: row.get::<_, Option<String>>(2)?.unwrap_or_default(), // Handle NULL values for name
                      description: row.get::<_, Option<String>>(3)?.unwrap_or_default(), // Handle NULL values for description
                      wikidata_id: row.get(1)?,
                      custom_properties,
                  })
            })?;
        
            let mut items = Vec::new();
            for row in rows {
                items.push(row?);
            }
        
            Ok(items)
        }

        async fn get_or_create_property(
            &self,
            tx: &mut rusqlite::Transaction<'_>,
            prop: &str,
        ) -> Result<i64, Error> {
            match tx.query_row("SELECT id FROM properties WHERE name = ?", [prop], |row| {
                row.get::<_, i64>(0)
            }) {
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
            let url_id = match tx.query_row("SELECT id FROM urls WHERE url = ?", [url], |row| {
                row.get::<_, i64>(0)
            }) {
                Ok(id) => {
                    log!("[DB] Found existing URL ID: {}", id);
                    id
                }
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
            let max_order: i32 = tx.query_row(
                "SELECT COALESCE(MAX(item_order), 0) FROM items WHERE url_id = ?",
                [url_id],
                |row| row.get(0),
            )?;

            log!("[DB] Upserting item");
            tx.execute(
                "INSERT INTO items (id, url_id, wikidata_id, item_order)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    url_id = excluded.url_id,
                    wikidata_id = excluded.wikidata_id",
                rusqlite::params![
                    &item.id,
                    url_id,
                    &item.wikidata_id,
                    max_order + 1
                ],
            )?;
            log!("[DB] Item upserted successfully");

            // property handling
            let core_properties = vec![
                ("name", &item.name),
                ("description", &item.description)
            ];

            for (prop, value) in core_properties.into_iter().chain(
                item.custom_properties.iter().map(|(k, v)| (k.as_str(), v))
            ) {
                let prop_id = self.get_or_create_property(&mut tx, prop).await?;
                
                tx.execute(
                    "INSERT INTO item_properties (item_id, property_id, value)
                    VALUES (?, ?, ?)
                    ON CONFLICT(item_id, property_id) DO UPDATE SET
                        value = excluded.value",
                    rusqlite::params![&item.id, prop_id, value],
                )?;
            }

            // Property synchronization
            log!("[DB] Synchronizing properties for item {}", item.id);
            let existing_props = {
                let mut stmt = tx.prepare(
                    "SELECT p.name, ip.value 
                    FROM item_properties ip
                    JOIN properties p ON ip.property_id = p.id
                    WHERE ip.item_id = ?",
                )?;
            
                let mapped_rows = stmt.query_map([&item.id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
            
                mapped_rows.collect::<Result<HashMap<String, String>, _>>()?
            };

            // Include core properties in current_props check
            let mut current_props: HashSet<&str> = item.custom_properties.keys()
                .map(|s| s.as_str())
                .collect();
            current_props.insert("name");
            current_props.insert("description");
            
            // Cleanup with core property protection
            for (existing_prop, _) in existing_props {
                if !current_props.contains(existing_prop.as_str()) 
                    && !["name", "description"].contains(&existing_prop.as_str())
                {
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
            let url_id: i64 =
                tx.query_row("SELECT id FROM urls WHERE url = ?", [url], |row| row.get(0))?;

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
            let url_id: i64 =
                tx.query_row("SELECT id FROM urls WHERE url = ?", [url], |row| row.get(0))?;

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

            // Insert URL if it does not exists
            tx.execute("INSERT OR IGNORE INTO urls (url) VALUES (?)", [url])?;

            // Get URL ID
            let url_id = tx.query_row("SELECT id FROM urls WHERE url = ?", [url], |row| {
                row.get::<_, i64>(0)
            })?;

            // Get/Create property
            let prop_id = match tx.query_row(
                "SELECT id FROM properties WHERE name = ?",
                [property],
                |row| row.get::<_, i64>(0),
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
                [url_id, prop_id],
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
                 WHERE u.url = ?",
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
                Ok(format!(
                    "ID: {}, URL: {}",
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?
                ))
            })?;
            for url in urls {
                log!("[DATABASE DEBUG] {}", url?);
            }

            log!("[DATABASE DEBUG] Items:");
            let mut stmt = conn.prepare("SELECT id, name FROM items")?;
            let items = stmt.query_map([], |row| {
                Ok(format!(
                    "ID: {}, Name: '{}'",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?
                ))
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
