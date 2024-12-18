/// Represents an Item in CompareWare.
/// Each item has metadata and key-value tags for categorization.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<(String, String)>,
    pub reviews: Vec<ReviewWithRating>,
    pub wikidata_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReviewWithRating {
    pub content: String,
    pub rating: u8, // Ratings from 1 to 5
}
