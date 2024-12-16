/// Main application entry point for CompareWare.
/// Combines the item management components (form and list) to provide a cohesive user interface.
use leptos::*;
use leptos_meta::*;
use crate::components::{item_form::ItemForm, items_list::ItemsList, review_form::ReviewForm, reviews_list::ReviewsList };
use crate::models::item::Item;
use crate::models::review::Review;
use crate::nostr::NostrClient;
use tokio::sync::mpsc;
use uuid::Uuid;
use leptos::spawn_local;
use nostr_sdk::serde_json;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // Signal to store and update the list of items.
    let (items_signal, set_items) = create_signal(Vec::<Item>::new());
    let (tx, mut rx) = mpsc::channel::<String>(100);

    spawn_local(async move {
        //initialize nostr client
        let nostr_client = NostrClient::new("wss://relay.damus.io").await.unwrap();
        nostr_client.subscribe_to_items(tx.clone()).await.unwrap();

        //handle incoming events
        while let Some(content) = rx.recv().await {
            if let Ok(item) = serde_json::from_str::<Item>(&content) {
                set_items.update(|items| items.push(item));
            }
        }
    });
    // Function to handle adding a new item to the list.
    let add_item = move |name: String, description: String, tags: Vec<(String, String)>| {
        set_items.update(|items| {
            let item = Item {
                id: Uuid::new_v4().to_string(),
                name: name.clone(),
                description: description.clone(),
                tags: tags.clone(),
                reviews: vec![],
            };
            items.push(item);
        });

        spawn_local(async move {
            let nostr_client = NostrClient::new("wss://relay.example.com").await.unwrap();
            nostr_client.publish_item(name, description, tags).await.unwrap();
        });
    };

    // Handle review submission
    let submit_review = move |content: String| {
        // Handle the review submission logic
    };

    view! {
        <>
            <Stylesheet href="/assets/style.css" />
            <div>
                <h1>{ "CompareWare" }</h1>
                // Form component for adding new items.
                <ItemForm on_submit=Box::new(add_item) />
                // Component to display the list of items.
                <ItemsList items=items_signal />
                // Reviews form and list
                <ReviewForm item_id={items_signal.get().first().unwrap().id.clone()} on_submit={submit_review} />
                <ReviewsList reviews={vec![]} /> 
            </div>
        </>
    }
}