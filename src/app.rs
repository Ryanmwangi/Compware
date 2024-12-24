use leptos::*;
use leptos_meta::*;
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
        <Stylesheet href="/assets/style.css" />
        <div>
            <h1>{ "CompareWare" }</h1>
            <ItemsList items=items_signal set_items=set_items />
        </div>
    }
}
