/// Component to display a list of items.
/// Iterates through the items and renders their name, description, tags, and reviews.
use leptos::*;
use crate::models::item::Item;
use serde::Deserialize;
use futures::future;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use std::sync::Arc;

// Define the structure for Wikidata API response
#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    entities: std::collections::HashMap<String, WikidataEntity>,
}
#[derive(Deserialize, Clone, Debug)]
struct WikidataEntity {
    labels: Option<std::collections::HashMap<String, WikidataLabel>>,
    descriptions: Option<std::collections::HashMap<String, WikidataLabel>>,
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataLabel {
    value: String,
}

#[component]
pub fn ItemsList(items: ReadSignal<Vec<Item>>) -> impl IntoView {
    // Create a signal for selected items
    let (selected_items_signal, set_selected_items) = create_signal(Vec::<usize>::new());
    let (wikidata_data, set_wikidata_data) = create_signal(Vec::<Option<WikidataEntity>>::new());

     // Fetch additional data from Wikidata for selected items
     let fetch_wikidata = move || {
        let selected_indices = selected_items_signal.get();
        let selected_items: Vec<Item> = selected_indices
            .iter()
            .map(|&i| items.get()[i].clone())
            .collect();

        
        // Wrap `selected_items` in an `Arc` so it can be cloned
        let selected_items = Arc::new(selected_items);

        // Clone selected_items before the async block to avoid borrowing issues
        let selected_items_clone: Arc<Vec<Item>> = Arc::clone(&selected_items);

        // For each selected item, fetch Wikidata attributes
        let futures = selected_items_clone.iter().map(move |item| {
            let wikidata_id = item.wikidata_id.clone();
            async move {
                if let Some(id) = wikidata_id {
                    let url = format!("https://www.wikidata.org/wiki/Special:EntityData/{}.json", id);
                    match Request::get(&url).send().await {
                        Ok(response) => match response.json::<WikidataResponse>().await {
                            Ok(parsed_data) => parsed_data.entities.get(&id).cloned(),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
        }).collect::<Vec<_>>();

    
        spawn_local(async move {
            let results = future::join_all(futures).await;
            set_wikidata_data.set(results);
        });
    };
    
    // Function to toggle selection of an item
    let toggle_selection = move |i: usize| {
        set_selected_items.update(|items| {
            if items.contains(&i) {
                items.retain(|&x| x != i);
            } else {
                items.push(i);
            }
        });
    };

    view! {
        <div>
            <h2>{ "Items" }</h2>
            <ul>
                {move || items.get().iter().enumerate().map(|(i, item)| view! {
                    <li key={i.to_string()}>
                        <label>
                            <input
                                type="checkbox"
                                checked={selected_items_signal.get().contains(&i)}
                                on:change=move |_| toggle_selection(i)
                             />
                             {"Select item for comparison"}
                        </label>
                        <div>
                            <strong>{ &item.name }</strong> - { &item.description }
                        </div>
                        <ul>
                            <h4>{ "Tags:" }</h4>
                            {item.tags.iter().map(|(key, value)| view! {
                                <li>{ key.clone() + ": " + value }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                        <ul>
                            <h4>{ "Reviews:" }</h4>
                            {item.reviews.iter().map(|review| view! {
                                <li>{ format!("Rating: {}/5 - {}", review.rating, review.content) }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                    </li>
                }).collect::<Vec<_>>() }
            </ul>

            // Button to fetch Wikidata attributes
            <button on:click=move |_| fetch_wikidata()>{ "Fetch External Data" }</button>

            // Comparison Table
            <h2>{ "Comparison Table" }</h2>
            <table>
                <thead>
                    <tr>
                        <th>{ "Item Name" }</th>
                        <th>{ "Description" }</th>
                        <th>{ "Tags" }</th>
                        <th>{ "Reviews" }</th>
                        <th>{ "External Description (Wikidata)" }</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let selected_indices = selected_items_signal.get();
                        selected_indices.iter().enumerate().map(|(idx, &i)| {
                            let item = &items.get()[i];
                            let wikidata_entity = wikidata_data.get().get(idx).cloned().flatten();
                            view! {
                                <tr key={i.to_string()}>
                                    <td>{ &item.name }</td>
                                    <td>{ &item.description }</td>
                                    <td>
                                        {item.tags.iter().map(|(key, value)| view! {
                                            <span>{ key.clone() + ": " + value + " " }</span>
                                        }).collect::<Vec<_>>()}
                                    </td>
                                    <td>
                                        {item.reviews.iter().map(|review| view! {
                                            <span>{ format!("Rating: {}/5 ", review.rating) }</span>
                                        }).collect::<Vec<_>>()}
                                    </td>
                                    <td>
                                        {move || {
                                            let entity = wikidata_entity.clone(); // Clone the value
                                            match entity {
                                                Some(entity) => entity
                                                    .descriptions
                                                    .as_ref()
                                                    .and_then(|descriptions| descriptions.get("en"))
                                                    .map(|label| label.value.clone())
                                                    .unwrap_or_default(),
                                                None => String::from("No description"),
                                            }
                                        }}
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}
