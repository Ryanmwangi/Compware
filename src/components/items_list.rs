use crate::components::editable_cell::EditableCell;
use crate::components::tag_editor::TagEditor;
use leptos::*;
use serde::Deserialize;
use uuid::Uuid;
use leptos::logging::log;
use crate::models::item::Item;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use web_sys::{FocusEvent, HtmlElement, Element, Node};
use futures_timer::Delay;
use std::time::Duration;

#[derive(Deserialize, Clone, Debug)]
struct WikidataSuggestion {
    id: String,
    label: String,
    description: Option<String>,
}
#[derive(Clone)]
struct ActiveCell {
    row_index: usize,
    position: (f64, f64),
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

    let (active_cell, set_active_cell) = create_signal(None::<ActiveCell>);
    let (active_cell_position, set_active_cell_position) = create_signal(None::<(f64, f64)>);
    let (active_row_index, set_active_row_index) = create_signal(None::<usize>);
    let (wikidata_suggestions, set_wikidata_suggestions) = create_signal(Vec::<WikidataSuggestion>::new());
    let debounce_duration = Duration::from_millis(300);

    // Fetch Wikidata suggestions
    let fetch_wikidata_suggestions = move |query: String| {
        spawn_local(async move {
            Delay::new(debounce_duration).await;
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
                Err(err) => {
                    log!("Failed to fetch Wikidata suggestions: {:?}", err);
                    set_wikidata_suggestions.set(Vec::new());
                }
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
                        set_active_row_index.set(Some(index));
                        // Set active cell position with validation
                        if let Some(element) = document().get_element_by_id(&format!("name-{}", index)) {
                            let rect = element.get_bounding_client_rect();
                            log!("Bounding rect: {:?}", rect); // Log rect details
                            if rect.width() > 0.0 && rect.height() > 0.0 {
                                set_active_cell_position.set(Some((rect.left(), rect.bottom())));
                            } else {
                                log!("Element bounding box is not valid for popup positioning.");
                            }
                        }
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

    // Position and render the popup
    let render_popup = move || {
            view! {
                <div
                    class="suggestions-popup"
                    style=move || {
                        let suggestions = wikidata_suggestions.get();
                        if !suggestions.is_empty() {
                            if let Some((x, y)) = active_cell_position.get() {
                                format!(
                                    "position: absolute; left: {}px; top: {}px; display: block; z-index: 1000;",
                                    x, y
                                )
                            } else {
                                "display: none;".to_string()
                            }
                        } else {
                            "display: none;".to_string()
                        }
                    }
                >
                    <ul>
                        {move || wikidata_suggestions.get().iter().map(|suggestion| {
                            let label_for_click = suggestion.label.clone();
                            let description_for_click = suggestion.description.clone().unwrap_or_default();
                            let id = suggestion.id.clone();
                            let label_for_display = label_for_click.clone();
                            let description_for_display = description_for_click.clone();

                            view! {
                                <li on:click=move |_| {
                                    if let Some(index) = active_row_index.get() {
                                        set_items.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.name = label_for_click.clone();
                                                item.description = description_for_click.clone();
                                                item.wikidata_id = Some(id.clone());
                                                item.tags.push(("wikidata_id".to_string(), id.clone()));
                                            }
                                        });
                                    }
                                    set_wikidata_suggestions.set(Vec::new());
                                    set_active_cell_position.set(None);
                                }>
                                    {format!("{} - {}", label_for_display, description_for_display)}
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                </div>
            }
        };
    
    view! {
        <div>
            <h1>{ "Items List" }</h1>
            <table>
                <thead>
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Description" }</th>
                        <th>{ "Tags" }</th>
                        <th>{ "Actions" }</th>
                    </tr>
                </thead>
                <tbody>
                    {move || items.get().iter().enumerate().map(|(index, item)| {
                        view! {
                            <tr>
                                // Editable Name Field with Wikidata Integration
                                <td>
                                    <div
                                        on:focus=move |event: FocusEvent| {
                                            if let Some(element) = event.target().and_then(|t| t.dyn_into::<HtmlElement>().ok()) {
                                                let rect = element.get_bounding_client_rect();
                                                set_active_cell.set(Some(ActiveCell {
                                                    row_index: index,
                                                    position: (rect.left(), rect.top() + rect.height()),
                                                }));
                                            }
                                        }
                                        on:blur=move |_| {
                                            spawn_local(async move {
                                                Delay::new(Duration::from_millis(100)).await;
                                                set_active_cell.set(None);
                                            });
                                        }                                        
                                    >
                                        <EditableCell
                                            value=item.name.clone()
                                            on_input=move |value| update_item(index, "name", value)
                                            key=format!("name-{}", index)
                                        />
                                    </div>
                                </td>
                                // Editable Description Field
                                <td>
                                    <EditableCell
                                        value=item.description.clone()
                                        on_input=move |value| update_item(index, "description", value)
                                        key=format!("description-{}", index)
                                    />
                                </td>
                                // Tag Editor
                                <td>
                                    <TagEditor
                                        tags=item.tags.clone()
                                        on_add=move |key, value| add_tag(index, key, value)
                                        on_remove=Arc::new(Mutex::new(move |tag_index: usize| remove_tag(index, tag_index)))
                                    />
                                </td>
                                // Actions
                                <td>
                                    <button on:click=move |_| remove_item(index)>{ "Delete" }</button>
                                </td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
            {render_popup()}
        </div>
    }
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    search: Vec<WikidataSuggestion>,
}