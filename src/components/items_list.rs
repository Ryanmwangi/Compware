use crate::models::item::Item;
use gloo_net::http::Request;
use leptos::logging::log;
use leptos::*;
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    entities: HashMap<String, WikidataEntity>,
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataEntity {
    labels: Option<HashMap<String, WikidataLabel>>,
    descriptions: Option<HashMap<String, WikidataLabel>>,
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataLabel {
    value: String,
}

#[component]
pub fn ItemsList(items: ReadSignal<Vec<Item>>, set_items: WriteSignal<Vec<Item>>, on_add_item: impl Fn() + 'static, ) -> impl IntoView {
    let (wikidata_data, set_wikidata_data) = create_signal(Vec::<Option<WikidataEntity>>::new());

    // Fetch data from Wikidata for a given item name
    let fetch_wikidata = move |item_name: String| {
        spawn_local(async move {
            let url = format!("https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&limit=1&format=json", item_name);
            match Request::get(&url).send().await {
                Ok(response) => match response.json::<WikidataResponse>().await {
                    Ok(parsed_data) => {
                        if let Some(entity) = parsed_data.entities.values().next() {
                            set_wikidata_data.update(|current_data| {
                                current_data.push(Some(entity.clone()));
                            });
                        }
                    }
                    Err(_) => log!("Failed to parse response from Wikidata"),
                },
                Err(_) => log!("Failed to make request to Wikidata"),
            }
        });
    };

    // Handle updating grid cells
    let update_item = move |index: usize, column: &str, value: String| {
        set_items.update(|items| {
            if let Some(item) = items.get_mut(index) {
                match column {
                    "name" => item.name = value,
                    "description" => item.description = value,
                    "tags" => item.tags.push((value.clone(), value)), // For simplicity, adding the same value as key and value.
                    "review" => item.reviews.push(crate::models::item::ReviewWithRating {
                        content: value.clone(),
                        rating: 5, // Assuming a default rating of 5
                    }),
                    "rating" => {
                        if let Ok(rating) = value.parse::<u8>() {
                            item.reviews.last_mut().map(|r| r.rating = rating);
                        }
                    }
                    _ => (),
                }
            }
        });
    };

    // Trigger add item event
    let add_item = move |_| {
        on_add_item(); // Call the passed closure from App to add a new item
    };

    view! {
        <div>
            <h1>{ "CompareWare" }</h1>

            <button on:click=add_item>{ "Add New Item" }</button>

            <table>
                <thead>
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Description" }</th>
                        <th>{ "Tags" }</th>
                        <th>{ "Review" }</th>
                        <th>{ "Rating" }</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        items.get().iter().enumerate().map(|(index, item)| {
                            view! {
                                <tr>
                                    <td>
                                        <input
                                            type="text"
                                            value={item.name.clone()}
                                            on:input=move |e| {
                                                update_item(index, "name", event_target_value(&e));
                                                fetch_wikidata(event_target_value(&e)); // Fetch Wikidata when name is entered
                                            }
                                        />
                                    </td>
                                    <td>
                                        <input
                                            type="text"
                                            value={item.description.clone()}
                                            on:input=move |e| update_item(index, "description", event_target_value(&e))
                                        />
                                    </td>
                                    <td>
                                        <input
                                            type="text"
                                            placeholder="Add tags"
                                            on:input=move |e| update_item(index, "tags", event_target_value(&e))
                                        />
                                    </td>
                                    <td>
                                        <textarea
                                            value={item.reviews.iter().map(|review| format!("{}: {}", review.rating, review.content)).collect::<Vec<_>>().join("\n")}
                                            on:input=move |e| update_item(index, "review", event_target_value(&e))
                                        />
                                    </td>
                                    <td>
                                        <input
                                            type="number"
                                            value={item.reviews.last().map(|r| r.rating).unwrap_or(5)}
                                            min="1" max="5"
                                            on:input=move |e| update_item(index, "rating", event_target_value(&e))
                                        />
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
