// src/models/review.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Review {
    pub item_id: String,     // ID of the item the review is associated with
    pub content: String,     // Content of the review
    pub user_id: String,     // ID of the user who submitted the review
}
