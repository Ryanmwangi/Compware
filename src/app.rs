use leptos::*;
use leptos_meta::*;
use leptos_router::*; 
use leptos::logging::log;
use crate::components::items_list::{ItemsList, load_items_from_db};
use crate::models::item::Item;
use leptos::spawn_local;
// use tokio::sync::mpsc;
// use crate::nostr::NostrClient;
// use nostr_sdk::serde_json;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Signal to manage the list of items
    let (items_signal, set_items) = create_signal(Vec::<Item>::new());
    // let (tx, mut rx) = mpsc::channel::<String>(100);

    // // Nostr client subscription for items
    // spawn_local(async move {
    //     let nostr_client = NostrClient::new("wss://relay.example.com").await.unwrap();
    //     nostr_client.subscribe_to_items(tx.clone()).await.unwrap();

    //     while let Some(content) = rx.recv().await {
    //         if let Ok(item) = serde_json::from_str::<Item>(&content) {
    //             set_items.update(|items| items.push(item));
    //         }
    //     }
    // });
    view! {
        <head>
            <script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
            <script src="https://twitter.github.io/typeahead.js/releases/latest/typeahead.bundle.min.js"></script>
        </head>
        <Router>
            <Routes>
                <Route path="/*url" view=move || {
                    let location = use_location();
                    let current_url = move || location.pathname.get();
                    
                    // Proper async handling
                    spawn_local({
                        let current_url = current_url.clone();
                        async move {
                            match load_items_from_db(&current_url()).await {
                                Ok(items) => set_items.set(items),
                                Err(e) => log!("Error loading items: {}", e),
                            }
                        }
                    });
                    view! {
                        <Stylesheet href="/assets/style.css" />
                        <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.1.1/css/all.min.css" />
                        <div>
                            <h1>{ "CompareWare" }</h1>
                            <ItemsList 
                            url=current_url() 
                            items=items_signal 
                            set_items=set_items />
                        </div>
                    }
                }/>
            </Routes>
        </Router>
    }
}
