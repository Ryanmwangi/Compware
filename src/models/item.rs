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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikidataSuggestion {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, rename = "display")]
    pub display: DisplayInfo,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DisplayInfo {
    #[serde(default, rename = "label")]
    pub label: LabelInfo,
    #[serde(default, rename = "description")]
    pub description: DescriptionInfo,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LabelInfo {
    #[serde(default, rename = "value")]
    pub value: String,
    #[serde(default, rename = "language")]
    pub language: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DescriptionInfo {
    #[serde(default, rename = "value")]
    pub value: String,
    #[serde(default, rename = "language")]
    pub language: String,
}