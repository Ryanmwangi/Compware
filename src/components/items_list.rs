/// Component to display a list of items.
/// Iterates through the items and renders their name, description, and tags.
use leptos::*;

use crate::models::item::Item;

#[component]
pub fn ItemsList(items: ReadSignal<Vec<Item>>) -> impl IntoView {
    view! {
        <div>
            <h2>{ "Items" }</h2>
            <ul>
            {move || items.get().iter().enumerate().map(|(i, item)| view! {
                    <li key={i.to_string()}>
                        <strong>{ item.name.clone() }</strong> - { item.description.clone() }
                        <ul>
                            {item.tags.iter().map(|(key, value)| view! {
                                <li>{ key.clone() + ": " + value }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                    </li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}