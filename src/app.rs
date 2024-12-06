use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::components::{item_form::ItemForm, items_list::ItemsList};
use crate::models::item::Item;
use std::sync::Arc;

#[component]
pub fn App() -> impl IntoView {
    let items = create_signal(Vec::<Item>::new());

    let add_item = move |name: String, description: String, tags: Vec<(String, String)>| {
        items.update(|items| {
            items.push(Item {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                description,
                tags,
            });
        });
    };

    view! {
        <div>
            <h1>CompareWare</h1>
            <ItemForm on_submit=add_item />
            <ItemsList items=items.get().clone() />
        </div>
    }
}