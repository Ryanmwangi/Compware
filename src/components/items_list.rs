use crate::components::editable_cell::EditableCell;
use crate::components::tag_editor::TagEditor;
use leptos::*;
use serde::Deserialize;
use uuid::Uuid;
use leptos::logging::log;
use crate::models::item::Item;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;

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
    // State to track the currently focused cell
    let (focused_cell, set_focused_cell) = create_signal(None::<String>);

    // State to manage dynamic property names
    let (custom_properties, set_custom_properties) = create_signal(Vec::<String>::new());
    
    // state to manage suggestions visibility
    let (show_suggestions, set_show_suggestions) = create_signal(false);

    // Ensure there's an initial empty row
    set_items.set(vec![Item {
        id: Uuid::new_v4().to_string(),
        name: String::new(),
        description: String::new(),
        tags: vec![],
        reviews: vec![],
        wikidata_id: None,
        custom_properties: HashMap::new(),
    }]);

    let (wikidata_suggestions, set_wikidata_suggestions) = create_signal(HashMap::<String, Vec<WikidataSuggestion>>::new());

    // Fetch Wikidata suggestions
    let fetch_wikidata_suggestions = move |key:String, query: String| {
        log!("Fetching suggestions for key: {}, query: {}", key, query);
        spawn_local(async move {
            if query.is_empty() {
                set_wikidata_suggestions.update(|suggestions| {
                    suggestions.remove(&key);
                });
                return;
            }

            let url = format!(
                "https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&limit=5&format=json&origin=*",
                query
            );

            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) => {
                    if let Ok(data) = response.json::<WikidataResponse>().await {
                        log!("Fetching suggestions for key: {}, query: {}", key, query);
                        set_wikidata_suggestions.update(|suggestions| {
                            log!("Updated suggestions: {:?}", suggestions);
                            suggestions.insert(key, data.search);
                        });
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
                        fetch_wikidata_suggestions(format!("name-{}", index), value.clone());
                    }
                    "description" => {
                        item.description = value.clone();
                    }
                    _ => {
                        // Update custom property
                        item.custom_properties.insert(field.to_string(), value.clone());
                    }
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
                    custom_properties: HashMap::new(),
                });
            }
        });
    };

    // Add a new custom property
    let add_property = move |property: String| {
        set_custom_properties.update(|props| {
            if !props.contains(&property) && !property.is_empty() {
                props.push(property);
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
                                                    <div class="editable-cell">
                                                        <EditableCell
                                                            value=item.name.clone()
                                                            on_input=move |value| {
                                                                update_item(index, "name", value.clone());
                                                                fetch_wikidata_suggestions(format!("name-{}", index), value);
                                                            }
                                                            key=Arc::new(format!("name-{}", index))
                                                            focused_cell=focused_cell
                                                            set_focused_cell=set_focused_cell.clone()
                                                            on_focus=Some(Callback::new(move |_| {
                                                                log!("Input focused, showing suggestions");
                                                                set_show_suggestions.set(true);
                                                            }))
                                                            on_blur=Some(Callback::new(move |_| {
                                                                log!("Input blurred, delaying hiding suggestions");
                                                                spawn_local(async move {
                                                                    gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
                                                                    log!("Hiding suggestions after delay");
                                                                    set_show_suggestions.set(false);
                                                                });
                                                            }))
                                                        />
                                                        {move || {
                                                            if show_suggestions.get() {
                                                                log!("Rendering suggestions list");
                                                                view! {
                                                                        <ul class="editable-cell-suggestions">
                                                                                {move || {
                                                                                    let suggestions = wikidata_suggestions.get()
                                                                                        .get(&format!("name-{}", index))
                                                                                        .cloned()
                                                                                        .unwrap_or_default();
                                                                                    log!("Suggestions for cell {}: {:?}", index, suggestions);
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
                                                                                    <li class="editable-cell-suggestions-li" on:click=move |_| {
                                                                                        set_items.update(|items| {
                                                                                            if let Some(item) = items.get_mut(index) {
                                                                                                        item.description = description_for_click.clone();
                                                                                                        item.tags.extend(tags.clone());
                                                                                                        item.wikidata_id = Some(id.clone());
                                                                                                        item.name = label_for_click.clone();
                                                                                            }
                                                                                        });
                                                                                        set_show_suggestions.set(false);
                                                                                    }>
                                                                                                { format!("{} - {}", label_for_display, description_for_display) }
                                                                                    </li>
                                                                                }
                                                                                    }).collect::<Vec<_>>()
                                                                                }}
                                                                        </ul>
                                                                }
                                                            } else {
                                                                log!("Suggestions list hidden");
                                                                view! {
                                                                    <ul></ul>
                                                                }
                                                            }
                                                        }}
                                                    </div>
                                                }.into_view(),
                                                "Description" => view! {
                                                <EditableCell
                                                    value=item.description.clone()
                                                    on_input=move |value| update_item(index, "description", value)
                                                    key=Arc::new(format!("description-{}", index))
                                                    focused_cell=focused_cell
                                                    set_focused_cell=set_focused_cell.clone()
                                                    on_focus=Some(Callback::new(move |_| {
                                                        log!("Description input focused");
                                                    }))
                                                    on_blur=Some(Callback::new(move |_| {
                                                        log!("Description input blurred");
                                                    }))
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
                    // Dynamically adding custom properties as columns
                    {move || {
                        let custom_props = custom_properties.get().clone();
                        custom_props.into_iter().map(move |property| {
                            let property_clone = property.clone();
                            view! {
                                <tr>
                                    <td>{ property }</td>
                                    {move || {
                                        let property_clone = property_clone.clone(); // Clone `property_clone` again for the inner closure
                                        items.get().iter().enumerate().map(move |(index, item)| {
                                            let property_clone_for_closure = property_clone.clone();
                                            view! {
                                                <td>
                                                    <EditableCell
                                                        value=item.custom_properties.get(&property_clone).cloned().unwrap_or_default()
                                                        on_input=move |value| update_item(index, &property_clone_for_closure, value)
                                                        key=Arc::new(format!("custom-{}-{}", property_clone, index))
                                                        focused_cell=focused_cell
                                                        set_focused_cell=set_focused_cell.clone()
                                                        on_focus=Some(Callback::new(move |_| {
                                                            log!("Custom property input focused");
                                                        }))
                                                        on_blur=Some(Callback::new(move |_| {
                                                            log!("Custom property input blurred");
                                                        }))
                                                    />
                                                </td>
                                            }
                                        }).collect::<Vec<_>>()
                                    }}
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
            <div style="margin-bottom: 20px;">
                <input type="text" id="new-property" placeholder="Add New Property"/>
                <button on:click=move |_| {
                    let property = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("new-property")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlInputElement>()
                        .unwrap()
                        .value();
                    add_property(property);
                }>{ "Add Property" }</button>
            </div>
        </div>
    }
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    search: Vec<WikidataSuggestion>,
}