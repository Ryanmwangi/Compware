use leptos::*;
use crate::components::{item_form::ItemForm, items_list::ItemsList};
use crate::models::item::Item;

#[component]
pub fn App() -> impl IntoView {
    let (items_signal, set_items) = create_signal(Vec::<Item>::new());

    let add_item = move |name: String, description: String, tags: Vec<(String, String)>| {
        set_items;(|mut items: Vec<Item>| {
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
            <ItemForm on_submit=Box::new(add_item) />
            <ItemsList items={items_signal.get().clone()} />
        </div>
    }
}