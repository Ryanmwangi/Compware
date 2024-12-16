/// Represents an Item in CompareWare.
/// Each item has metadata and key-value tags for categorization.
use serde::{Deserialize, Serialize};
use crate::models::review::Review;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<(String, String)>,
    pub reviews: Vec<Review>, // Add reviews field here
}