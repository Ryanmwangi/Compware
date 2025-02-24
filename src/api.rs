#[cfg(feature = "ssr")]
use actix_web::{web, HttpResponse};
#[cfg(feature = "ssr")]
use crate::db::Database;
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;
#[cfg(feature = "ssr")]
use crate::models::item::Item;
#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use leptos::logging::log;

#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
#[derive(Serialize, Deserialize)]
pub struct ItemRequest {
    pub url: String,
    pub item: Item,
}

#[cfg(feature = "ssr")]
pub async fn get_items(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Query<String>,
) -> HttpResponse {
    log!("[SERVER] Received request for URL: {}", url);

    let db = db.lock().await;
    match db.get_items_by_url(&url).await {
        Ok(items) => {
            log!("[SERVER] Returning {} items for URL: {}", items.len(), url);
            HttpResponse::Ok().json(items)
        },
        Err(err) => {
            log!("[SERVER ERROR] Failed to fetch items for {}: {:?}", url, err);
            HttpResponse::InternalServerError().body("Failed to fetch items")
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn create_item(
    db: web::Data<Arc<Mutex<Database>>>,
    request: web::Json<ItemRequest>,
) -> HttpResponse {
    let db = db.lock().await;
    let url = request.url.clone();
    let item = request.item.clone();
    let item_id = request.item.id.clone();
    // request logging
    log!("[API] Received item request - URL: {}, Item ID: {}", 
        request.url, request.item.id);
    
    // raw JSON logging
    let raw_json = serde_json::to_string(&request.into_inner()).unwrap();
    log!("[API] Raw request JSON: {}", raw_json);

    match db.insert_item_by_url(&url, &item).await {
        Ok(_) => {
            log!("[API] Successfully saved item ID: {}", item_id);
            HttpResponse::Ok().json(item)
        },
        Err(e) => {
            log!("[API] Database error: {:?}", e); 
            HttpResponse::BadRequest().body(format!("Database error: {}", e))
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn delete_item(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Query<String>,
    item_id: web::Path<String>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.delete_item_by_url(&url, &item_id).await {
        Ok(_) => HttpResponse::Ok().body("Item deleted"),
        Err(err) => {
            leptos::logging::error!("Failed to delete item: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to delete item")
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn delete_property(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Query<String>,
    property: web::Path<String>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.delete_property_by_url(&url, &property).await {
        Ok(_) => HttpResponse::Ok().body("Property deleted"),
        Err(err) => {
            leptos::logging::error!("Failed to delete property: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to delete property")
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn get_items_by_url(
    db: web::Data<Arc<Mutex<Database>>>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let url = query.get("url").unwrap_or(&String::new()).to_string();
    let db = db.lock().await;
    match db.get_items_by_url(&url).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(err) => {
            leptos::logging::error!("Failed to fetch items by URL: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to fetch items by URL")
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn delete_item_by_url(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
    item_id: web::Path<String>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.delete_item_by_url(&url, &item_id).await {
        Ok(_) => HttpResponse::Ok().body("Item deleted"),
        Err(err) => {
            leptos::logging::error!("Failed to delete item by URL: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to delete item by URL")
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn delete_property_by_url(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
    property: web::Path<String>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.delete_property_by_url(&url, &property).await {
        Ok(_) => HttpResponse::Ok().body("Property deleted"),
        Err(err) => {
            leptos::logging::error!("Failed to delete property by URL: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to delete property by URL")
        }
    }
}