#[cfg(feature = "ssr")]
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use tokio::sync::Mutex;
#[cfg(feature = "ssr")]
use compareware::db::Database;
#[cfg(feature = "ssr")]
use compareware::api::{ItemRequest, create_item, get_items, get_selected_properties, add_selected_property};
#[cfg(feature = "ssr")]
use compareware::models::item::Item;
use compareware::utils::panic_hook;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use compareware::app::*;
    use compareware::db::Database;
    use compareware::api::{delete_item, delete_property}; // Import API handlers
    use std::sync::Arc;
    use tokio::sync::Mutex;

    panic_hook::init();
    
    // Setup logging
    std::env::set_var("RUST_LOG", "info");
    
    // Initialize the database
    let db = Database::new("compareware.db").unwrap();
    db.create_schema().await.unwrap(); // Ensure the schema is created
    let db = Arc::new(Mutex::new(db)); // Wrap the database in an Arc<Mutex<T>> for shared state
    println!("Schema created successfully!");
    
    // Load configuration 
    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let addr = conf.leptos_options.site_addr;

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    // Start the Actix Web server
    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        let db = db.clone(); // Clone the Arc for each worker


        App::new()
            .app_data(web::Data::new(db.clone()))
            // Register custom API routes BEFORE Leptos server functions
            .service(
                web::scope("/api")
                .service(
                    web::scope("/urls/{url}")
                        .route("/items", web::get().to(get_items_handler)) // GET items by URL
                        .route("/items", web::post().to(create_item_handler)) // Create item for URL
                        .route("/items/{item_id}", web::delete().to(delete_item)) // Delete item for URL
                        .route("/properties", web::get().to(get_selected_properties_handler))
                        .route("/properties", web::post().to(add_selected_property_handler))
                        .route("/properties/{property}", web::delete().to(delete_property)) // Delete property for URL
                )
            )
            // Register server functions
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // Serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // Serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            // Serve the favicon from /favicon.ico
            .service(favicon)
            // Register Leptos routes
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
            // Pass Leptos options to the app
            .app_data(web::Data::new(leptos_options.to_owned()))
            //.wrap(middleware::Compress::default())
            // Pass the database as shared state
            .app_data(web::Data::new(db))
            // Register URL routing
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/{url}").route(web::get().to(url_handler)))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
// Handler to get items for a specific URL
async fn get_items_handler(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
) -> impl Responder {
    get_items(db, web::Query(url.into_inner())).await
}

#[cfg(feature = "ssr")]
// Handler to create an item for a specific URL
async fn create_item_handler(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
    item: web::Json<Item>,
) -> impl Responder {
    let request = ItemRequest { 
        url: url.into_inner(),
        item: item.into_inner() 
    };
    create_item(db, web::Json(request)).await
}

// // Handler to delete an item for a specific URL
// async fn delete_item_handler(
//     db: web::Data<Arc<Mutex<Database>>>,
//     path: web::Path<(String, String)>,
// ) -> impl Responder {
//     let (url, item_id) = path.into_inner();
//     delete_item_by_url(db, web::Path::from(url), web::Path::from(item_id)).await
// }

#[cfg(feature = "ssr")]
async fn get_selected_properties_handler(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
) -> impl Responder {
    get_selected_properties(db, url).await
}

#[cfg(feature = "ssr")]
async fn add_selected_property_handler(
    db: web::Data<Arc<Mutex<Database>>>,
    url: web::Path<String>,
    property: web::Json<String>,
) -> impl Responder {
    add_selected_property(db, url, property).await
}

#[cfg(feature = "ssr")]
// Define the index handler
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Welcome to CompareWare!")
}

#[cfg(feature = "ssr")]
// Define the URL handler
async fn url_handler(url: web::Path<String>) -> HttpResponse {
    let url = url.into_inner();
    // TO DO: Implement URL-based content storage and editing functionality
    HttpResponse::Ok().body(format!("You are viewing the content at {}", url))
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // Initialize custom panic hook for better diagnostics
    panic_hook::init();

    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use compareware::app::*;

    // console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}