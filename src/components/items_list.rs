/// Component to display a list of items.
/// Iterates through the items and renders their name, description, tags, and reviews.
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
                        <h4>{ "Tags:" }</h4>
                        <ul>
                            {item.tags.iter().map(|(key, value)| view! {
                                <li>{ format!("{}: {}", key, value) }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                        <h4>{ "Reviews:" }</h4>
                        <ul>
                            {item.reviews.iter().map(|review| view! {
                                <li>{ review.clone() }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                    </li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}
