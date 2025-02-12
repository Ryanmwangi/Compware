/// Represents an Item in CompareWare.
/// Each item has metadata and key-value tags for categorization.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub wikidata_id: Option<String>,
    pub custom_properties: HashMap<String, String>,
}
