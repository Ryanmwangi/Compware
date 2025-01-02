use crate::components::editable_cell::EditableCell;
use crate::components::tag_editor::TagEditor;
use leptos::*;
use serde::Deserialize;
use uuid::Uuid;
use leptos::logging::log;
use crate::models::item::Item;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Clone, Debug)]
struct WikidataSuggestion {
    id: String,
    label: String,
    description: Option<String>,
}

#[component]
pub fn ItemsList(
    items: ReadSignal<Vec<Item>>,
    set_items: WriteSignal<Vec<Item>>,
) -> impl IntoView {
    
    // Ensure there's an initial empty row
    set_items.set(vec![Item {
        id: Uuid::new_v4().to_string(),
        name: String::new(),
        description: String::new(),
        tags: vec![],
        reviews: vec![],
        wikidata_id: None,
    }]);

    let (wikidata_suggestions, set_wikidata_suggestions) =
        create_signal(Vec::<WikidataSuggestion>::new());

    // Fetch Wikidata suggestions
    let fetch_wikidata_suggestions = move |query: String| {
        spawn_local(async move {
            if query.is_empty() {
                set_wikidata_suggestions.set(Vec::new());
                return;
            }

            let url = format!(
                "https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&limit=5&format=json&origin=*",
                query
            );

            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) => {
                    if let Ok(data) = response.json::<WikidataResponse>().await {
                        set_wikidata_suggestions.set(data.search);
                    }
                }
                Err(_) => log!("Failed to fetch Wikidata suggestions"),
            }
        });
    };

    // Update item fields
    let update_item = move |index: usize, field: &str, value: String| {
        set_items.update(|items| {
            if let Some(item) = items.get_mut(index) {
                match field {
                    "name" => {
                        item.name = value.clone();
                        fetch_wikidata_suggestions(value.clone());
                    }
                    "description" => {
                        item.description = value.clone();
                    }
                    _ => (),
                }
            }

            // Automatically add a new row when editing the last row
            if index == items.len() - 1 && !value.is_empty() {
                items.push(Item {
                    id: Uuid::new_v4().to_string(),
                    name: String::new(),
                    description: String::new(),
                    tags: vec![],
                    reviews: vec![],
                    wikidata_id: None,
                });
            }
        });
    };

    // Add a new tag to an item
    let add_tag = move |index: usize, key: String, value: String| {
        set_items.update(|items| {
            if let Some(item) = items.get_mut(index) {
                item.tags.push((key, value));
            }
        });
    };

    // Remove a tag from an item
    let remove_tag = move |item_index: usize, tag_index: usize| {
        set_items.update(|items| {
            if let Some(item) = items.get_mut(item_index) {
                item.tags.remove(tag_index);
            }
        });
    };

    // Remove an item
    let remove_item = move |index: usize| {
        set_items.update(|items| {
            items.remove(index);
        });
    };

    // List of properties to display as rows
    let properties = vec!["Name", "Description", "Tags", "Actions"];

    view! {
        <div>
            <h1>{ "Items List" }</h1>
            <table>
                <thead>
                    <tr>
                        <th>{ "Property" }</th>
                        {move || items.get().iter().enumerate().map(|(index, _)| {
                            view! {
                                <th>{ format!("Item {}", index + 1) }</th>
                            }
                        }).collect::<Vec<_>>()} 
                    </tr>
                </thead>
                <tbody>
                    {properties.into_iter().map(|property| {
                        view! {
                            <tr>
                                <td>{ property }</td>
                                {move || items.get().iter().enumerate().map(|(index, item)| {
                                    view! {
                                        <td>
                                            {match property {
                                                "Name" => view! {
                                                    <EditableCell
                                                        value=item.name.clone()
                                                        on_input=move |value| update_item(index, "name", value)
                                                        key=format!("name-{}", index)
                                                    />
                                                    <ul>
                                                        {move || {
                                                            let suggestions = wikidata_suggestions.get().to_vec();
                                                            suggestions.into_iter().map(|suggestion| {
                                                                let label_for_click = suggestion.label.clone();
                                                                let label_for_display = suggestion.label.clone();
                                                                let description_for_click = suggestion.description.clone().unwrap_or_default();
                                                                let description_for_display = suggestion.description.clone().unwrap_or_default();
                                                                let id = suggestion.id.clone();

                                                                // Tags for the item
                                                                let tags = vec![
                                                                    ("source".to_string(), "wikidata".to_string()),
                                                                    ("wikidata_id".to_string(), id.clone()),
                                                                ];

                                                                view! {
                                                                    <li on:click=move |_| {
                                                                        set_items.update(|items| {
                                                                            if let Some(item) = items.get_mut(index) {
                                                                                item.description = description_for_click.clone();
                                                                                item.tags.extend(tags.clone());
                                                                                item.wikidata_id = Some(id.clone());
                                                                                item.name = label_for_click.clone();
                                                                            }
                                                                        });
                                                                    }>
                                                                        { format!("{} - {}", label_for_display, description_for_display) }
                                                                    </li>
                                                                }
                                                            }).collect::<Vec<_>>()
                                                        }}
                                                    </ul>
                                                }.into_view(),
                                                "Description" => view! {
                                                    <EditableCell
                                                        value=item.description.clone()
                                                        on_input=move |value| update_item(index, "description", value)
                                                        key=format!("description-{}", index)
                                                    />
                                                }.into_view(),
                                                "Tags" => view! {
                                                    <TagEditor
                                                        tags=item.tags.clone()
                                                        on_add=move |key, value| add_tag(index, key, value)
                                                        on_remove=Arc::new(Mutex::new(move |tag_index: usize| remove_tag(index, tag_index)))
                                                    />
                                                }.into_view(),
                                                "Actions" => view! {
                                                    <button on:click=move |_| remove_item(index)>{ "Delete" }</button>
                                                }.into_view(),
                                                _ => view! {
                                                    { "" }
                                                }.into_view(),
                                            }}
                                        </td>
                                    }
                                }).collect::<Vec<_>>()}                                
                            </tr>
                        }
                    }).collect::<Vec<_>>()}                    
                </tbody>
            </table>
        </div>
    }
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    search: Vec<WikidataSuggestion>,
}