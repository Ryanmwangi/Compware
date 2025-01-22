#[cfg(feature = "ssr")]
mod db_impl {
    use rusqlite::{Connection, Error};
    use serde::{Deserialize, Serialize};

    // Define a struct to represent a database connection
    #[derive(Debug)]
    pub struct Database {
        conn: Connection,
    }

    impl Database {
        // Create a new database connection
        pub fn new(db_path: &str) -> Result<Self, Error> {
            let conn = Connection::open(db_path)?;
            Ok(Database { conn })
        }

        // Create the database schema
        pub fn create_schema(&self) -> Result<(), Error> {
            self.conn.execute_batch("
                CREATE TABLE IF NOT EXISTS items (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    wikidata_id TEXT,
                    custom_properties TEXT
                );
            ")?;
            Ok(())
        }

        // Insert a new item into the database
        pub fn insert_item(&self, item: &DbItem) -> Result<(), Error> {
            let wikidata_id = item.wikidata_id.as_ref().map(|s| s.as_str()).unwrap_or("");
            self.conn.execute(
                "INSERT INTO items (id, name, description, wikidata_id, custom_properties) VALUES (?, ?, ?, ?, ?);",
                &[&item.id, &item.name, &item.description, &wikidata_id.to_string(), &item.custom_properties],
            )?;
            Ok(())
        }

        // Retrieve all items from the database
        pub fn get_items(&self) -> Result<Vec<DbItem>, Error> {
            let mut stmt = self.conn.prepare("SELECT * FROM items;")?;
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
}

#[cfg(feature = "ssr")]
pub use db_impl::{Database, DbItem};