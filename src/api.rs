#[cfg(feature = "ssr")]
use actix_web::{web, HttpResponse};
#[cfg(feature = "ssr")]
use crate::db::{Database, DbItem};
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;

use serde::Deserialize;
#[cfg(feature = "ssr")]
#[derive(Deserialize)]
pub struct ItemRequest {
    pub url: String,
    pub item: DbItem,
}

#[cfg(feature = "ssr")]
pub async fn get_items(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Query<String>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.get_items_by_url(&url).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(err) => {
            leptos::logging::error!("Failed to fetch items: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to fetch items")
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn create_item(
    db: web::Data<Arc<Mutex<Database>>>,
    request: web::Json<ItemRequest>,
) -> HttpResponse {
    match db.lock().await.insert_item_by_url(&request.url, &request.item).await {
        Ok(_) => HttpResponse::Ok().body("Item created"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
    url: web::Path<String>,
) -> HttpResponse {
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
pub async fn create_item_by_url(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
    item: web::Json<DbItem>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.insert_item_by_url(&url, &item.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("Item inserted"),
        Err(err) => {
            leptos::logging::error!("Failed to insert item by URL: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to insert item by URL")
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