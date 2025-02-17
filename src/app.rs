use leptos::*;
use leptos_meta::*;
use leptos_router::*; 
use leptos::logging::log;
use crate::components::items_list::ItemsList;
use crate::models::item::Item;
use crate::nostr::NostrClient;
use tokio::sync::mpsc;
use leptos::spawn_local;
use nostr_sdk::serde_json;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Signal to manage the list of items
    let (items_signal, set_items) = create_signal(Vec::<Item>::new());
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Nostr client subscription for items
    spawn_local(async move {
        let nostr_client = NostrClient::new("wss://relay.example.com").await.unwrap();
        nostr_client.subscribe_to_items(tx.clone()).await.unwrap();

        while let Some(content) = rx.recv().await {
            if let Ok(item) = serde_json::from_str::<Item>(&content) {
                set_items.update(|items| items.push(item));
            }
        }
    });
    view! {
        <Router>
            <Routes>
                <Route path="/:url?" view=move || {
                    let params = use_params_map();
                    let url = move || params.with(|params| params.get("url").cloned().unwrap_or_default());
                    
                    // This effect will re-run when URL changes
                    create_effect(move |_| {
                        let current_url = url();
                        spawn_local(async move {
                            // Load items for new URL
                            match load_items_from_db(&current_url).await {
                                Ok(loaded_items) => {
                                    set_items.set(loaded_items);
                                }
                                Err(err) => log!("Error loading items: {}", err),
                            }
                        });
                    });
                        view! {
                            <Stylesheet href="/assets/style.css" />
                            <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.1.1/css/all.min.css" />
                            <div>
                                <h1>{ "CompareWare" }</h1>
                                <ItemsList items=items_signal set_items=set_items />
                            </div>
                        }
                }/>
            </Routes>
        </Router>
    }
}
