#[cfg(feature = "ssr")]
use actix_web::{web, HttpResponse};
#[cfg(feature = "ssr")]
use crate::db::{Database, DbItem};
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;

#[cfg(feature = "ssr")]
pub async fn get_items(db: web::Data<Arc<Mutex<Database>>>) -> HttpResponse {
    let db = db.lock().await;
    match db.get_items().await {
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
    item: web::Json<DbItem>,
) -> HttpResponse {
    let db = db.lock().await;
    match db.insert_item(&item.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("Item inserted"),
        Err(err) => {
            leptos::logging::error!("Failed to insert item: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to insert item")
        }
    }
}