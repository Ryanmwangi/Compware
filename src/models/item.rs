/// Represents an Item in CompareWare.
/// Each item has metadata and key-value tags for categorization.
use serde::{Deserialize, Serialize};

pub struct Review {
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,           // Unique ID for the item
    pub name: String,         // Item name
    pub description: String,  // Short description of the item
    pub tags: Vec<(String, String)>, // Key-value tags (e.g., "type" -> "software")
    pub reviews: Vec<Review>, // Reviews
}
