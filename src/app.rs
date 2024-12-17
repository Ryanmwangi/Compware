use leptos::*;
use leptos_meta::*;
use crate::components::{item_form::ItemForm, items_list::ItemsList};
use crate::models::item::{Item, ReviewWithRating};
use crate::nostr::NostrClient;
use tokio::sync::mpsc;
use uuid::Uuid;
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
        let nostr_client = NostrClient::new("wss://relay.damus.io").await.unwrap();
        nostr_client.subscribe_to_items(tx.clone()).await.unwrap();

        while let Some(content) = rx.recv().await {
            if let Ok(item) = serde_json::from_str::<Item>(&content) {
                set_items.update(|items| items.push(item));
            }
        }
    });

    // Add a new item and review using the unified form
    let add_item = move |name: String, description: String, tags: Vec<(String, String)>, review: String, rating: u8| {
        let new_id = Uuid::new_v4().to_string();
    
        set_items.update(|items| {
            let item = Item {
                id: new_id.clone(),
                name,
                description,
                tags,
                reviews: vec![ReviewWithRating { content: review.clone(), rating }],
            };
            items.push(item);
        });
    
        spawn_local(async move {
            let nostr_client = NostrClient::new("wss://relay.example.com").await.unwrap();
            nostr_client.publish_item("New item added!".to_string(), "".to_string(), vec![]).await.unwrap();
        });
    };
    
    view! {
        <Stylesheet href="/assets/style.css" />
        <div>
            <h1>{ "CompareWare" }</h1>
            // Unified form for adding an item and its first review
            <ItemForm on_submit=Box::new(add_item) />
            // Display all items, including reviews
            <ItemsList items=items_signal />
        </div>
    }
}
