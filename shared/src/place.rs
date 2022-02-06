use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceDocument {
    pub id: String,
    pub place_name: String,
    pub category_name: String,
    pub category_group_code: String,
    pub category_group_name: String,
    pub phone: String,
    pub address_name: String,
    pub road_address_name: String,
    pub x: String,
    pub y: String,
    pub place_url: String,
    pub distance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceSearchMetadata {
    pub total_count: i32,
    pub pageable_count: i32,
    pub is_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceSearchResult {
    pub meta: PlaceSearchMetadata,
    pub documents: Vec<PlaceDocument>,
}
