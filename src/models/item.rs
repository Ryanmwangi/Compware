use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,           // Unique ID for the item
    pub name: String,         // Item name
    pub description: String,  // Short description of the item
    pub tags: Vec<(String, String)>, // Key-value tags (e.g., "type" -> "software")
}
