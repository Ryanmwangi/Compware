/// Main application entry point for CompareWare.
/// Combines the item management components (form and list) to provide a cohesive user interface.
use leptos::*;
use crate::components::{item_form::ItemForm, items_list::ItemsList};
use crate::models::item::Item;
use uuid::Uuid;

#[component]
pub fn App() -> impl IntoView {
    // Signal to store and update the list of items.
    let (items_signal, set_items) = create_signal(Vec::<Item>::new());

    // Function to handle adding a new item to the list.
    let add_item = move |name: String, description: String, tags: Vec<(String, String)>| {
        set_items.update(|items| {
            items.push(Item {
                id: Uuid::new_v4().to_string(),
                name,
                description,
                tags,
            });
        });
    };

    view! {
        <div>
            <h1>{ "CompareWare" }</h1>
            // Form component for adding new items.
            <ItemForm on_submit=Box::new(add_item) />
            // Component to display the list of items.
            <ItemsList items=items_signal.get() />
        </div>
    }
}
