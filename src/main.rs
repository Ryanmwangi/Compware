#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use compareware::app::*;
    use compareware::db::{Database, DbItem};
    use compareware::api::{get_items, create_item}; // Import API handlers
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Load configuration
    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;

    // Initialize the database
    let db = Database::new("compareware.db").unwrap();
    db.create_schema().await.unwrap(); // Ensure the schema is created
    let db = Arc::new(Mutex::new(db)); // Wrap the database in an Arc<Mutex<T>> for shared state

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    // Start the Actix Web server
    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        let db = db.clone(); // Clone the Arc for each worker


        App::new()
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
            // Register API endpoints
            .route("/api/items", web::get().to(get_items))
            .route("/api/items", web::post().to(create_item))
    })
    .bind(&addr)?
    .run()
    .await
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
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use compareware::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}