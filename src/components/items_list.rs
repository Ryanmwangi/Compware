use crate::components::editable_cell::EditableCell;
use crate::components::editable_cell::InputType;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use leptos::logging::log;
use crate::models::item::Item;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::JsCast;

#[derive(Deserialize, Clone, Debug)]
struct WikidataSuggestion {
    id: String,
    label: String,
    description: Option<String>,
}


#[derive(Deserialize, Debug)]
struct DbItem {
    id: String,
    name: String,
    description: String,
    wikidata_id: Option<String>,
    custom_properties: String,
}

#[component]
pub fn ItemsList(
    items: ReadSignal<Vec<Item>>,
    set_items: WriteSignal<Vec<Item>>,
) -> impl IntoView {
    // State to track selected properties
    let (selected_properties, set_selected_properties) = create_signal(HashMap::<String, bool>::new());
    
    // State to track the currently focused cell
    let (focused_cell, set_focused_cell) = create_signal(None::<String>);

    // State to manage dynamic property names
    let (custom_properties, set_custom_properties) = create_signal(Vec::<String>::new());
    
    // state to manage suggestions visibility
    let (show_suggestions, set_show_suggestions) = create_signal(HashMap::<String, bool>::new());
    
    // cache to store fetched properties
    let (fetched_properties, set_fetched_properties) = create_signal(HashMap::<String, String>::new());
   
    // Signal to store the fetched property labels
    let (property_labels, set_property_labels) = create_signal(HashMap::<String, String>::new());
    
    spawn_local(async move {
        match load_items_from_db().await {
            Ok(loaded_items) => {
                // Set the loaded items
                if loaded_items.is_empty() {
                    // Initialize with one empty item if the database is empty
                    set_items.set(vec![Item {
                        id: Uuid::new_v4().to_string(),
                        name: String::new(),
                        description: String::new(),
                        wikidata_id: None,
                        custom_properties: HashMap::new(),
                    }]);
                } else {
                    set_items.set(loaded_items.clone());
                }
    
                // Derive selected properties from the loaded items
                let mut selected_props = HashMap::new();
                let loaded_items_clone = loaded_items.clone();
                for item in loaded_items {
                    for (property, _) in item.custom_properties {
                        selected_props.insert(property, true);
                    }
                }
                set_selected_properties.set(selected_props);

                // Update the custom_properties signal
                let mut custom_props = Vec::new();
                for item in loaded_items_clone {
                    for (property, _) in &item.custom_properties {
                        if !custom_props.iter().any(|p| p == property) {
                            custom_props.push(property.clone());
                        }
                    }
                }

                let custom_props_clone = custom_props.clone();
                set_custom_properties.set(custom_props);

                // Fetch labels for the custom properties
                let property_ids = custom_props_clone;
                let labels = fetch_property_labels(property_ids).await;
                set_property_labels.update(|labels_map| {
                    for (key, value) in labels {
                        labels_map.insert(key, value);
                    }
                });

                log!("Items after loading: {:?}", items.get());
            }
            Err(err) => {
                log!("Error loading items: {}", err);
            }
        }
    });


    // Ensure there's an initial empty row
    if items.get().is_empty() {
        set_items.set(vec![Item {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            description: String::new(),
            // reviews: vec![],
            wikidata_id: None,
            custom_properties: HashMap::new(),
        }]);
    }
    
    // Function to send an item to the backend API
    async fn save_item_to_db(item: Item, selected_properties: ReadSignal<HashMap<String, bool>>) {
        // Use a reactive closure to access `selected_properties`
        let custom_properties: HashMap<String, String> = (move || {
            let selected_props = selected_properties.get(); // Access the signal inside a reactive closure
            item.custom_properties
                .into_iter()
                .filter(|(key, _)| selected_props.contains_key(key)) // Use the extracted value
                .collect()
        })(); 

        // Serialize `custom_properties` to a JSON string
        let custom_properties = serde_json::to_string(&custom_properties).unwrap();
    
        // Create a new struct to send to the backend
        #[derive(Serialize, Debug)]
        struct ItemToSend {
            id: String,
            name: String,
            description: String,
            wikidata_id: Option<String>,
            custom_properties: String, // JSON-encoded string
        }
    
        let item_to_send = ItemToSend {
            id: item.id,
            name: item.name,
            description: item.description,
            wikidata_id: item.wikidata_id,
            custom_properties, // Use the serialized string
        };
    
        let response = gloo_net::http::Request::post("/api/items")
            .json(&item_to_send)
            .unwrap()
            .send()
            .await;
    
        match response {
            Ok(resp) => {
                if resp.status() == 200 {
                    log!("Item saved to database: {:?}", item_to_send);
                } else {
                    log!("Failed to save item: {}", resp.status_text());
                }
            }
            Err(err) => log!("Failed to save item: {:?}", err),
        }
    }

    //function to load items from database
    async fn load_items_from_db() -> Result<Vec<Item>, String> {
        let response = gloo_net::http::Request::get("/api/items")
            .send()
            .await
            .map_err(|err| format!("Failed to fetch items: {:?}", err))?;
    
        if response.status() == 200 {
            // Deserialize into Vec<DbItem>
            log!("Loading items from DB...");
            let db_items = response
                .json::<Vec<DbItem>>()
                .await
                .map_err(|err| format!("Failed to parse items: {:?}", err))?;

            log!("Deserialized DB items: {:?}", db_items);
    
            // Convert DbItem to Item
            let items = db_items
                .into_iter()
                .map(|db_item| {
                    // Deserialize `custom_properties` from a JSON string to a HashMap
                    let custom_properties: HashMap<String, String> =
                        serde_json::from_str(&db_item.custom_properties)
                            .unwrap_or_default(); // Fallback to an empty HashMap if deserialization fails
                    
                    log!("Loaded item: {:?}", db_item.id);
                    log!("Custom properties: {:?}", custom_properties);
                    
                    Item {
                        id: db_item.id,
                        name: db_item.name,
                        description: db_item.description,
                        wikidata_id: db_item.wikidata_id,
                        custom_properties, // Deserialized HashMap
                    }
                })
                .collect();
            log!("Converted items: {:?}", items);
            Ok(items)
        } else {
            Err(format!("Failed to fetch items: {}", response.status_text()))
        }
    }

    // remove an item
    let remove_item = move |index: usize| {
        let item_id = items.get()[index].id.clone();
        spawn_local(async move {
            let response = gloo_net::http::Request::delete(&format!("/api/items/{}", item_id))
                .send()
                .await;
            match response {
                Ok(resp) => {
                    if resp.status() == 200 {
                        set_items.update(|items| {
                            items.remove(index);
                        });
                        log!("Item deleted: {}", item_id);
                    } else {
                        log!("Failed to delete item: {}", resp.status_text());
                    }
                }
                Err(err) => log!("Failed to delete item: {:?}", err),
            }
        });
    };

    let remove_property = move |property: String| {
        spawn_local(async move {
            let response = gloo_net::http::Request::delete(&format!("/api/properties/{}", property))
                .send()
                .await;
            match response {
                Ok(resp) => {
                    if resp.status() == 200 {
                        set_custom_properties.update(|props| {
                            props.retain(|p| p != &property);
                        });
                        set_selected_properties.update(|selected| {
                            selected.remove(&property);
                        });
                        set_items.update(|items| {
                            for item in items {
                                item.custom_properties.remove(&property);
                            }
                        });
                        log!("Property deleted: {}", property);
                    } else {
                        log!("Failed to delete property: {}", resp.status_text());
                    }
                }
                Err(err) => log!("Failed to delete property: {:?}", err),
            }
        });
    };

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

    //function to fetch properties
    async fn fetch_item_properties(wikidata_id: &str, set_fetched_properties: WriteSignal<HashMap<String, String>>, set_property_labels: WriteSignal<HashMap<String, String>>,) -> HashMap<String, String> {
        let url = format!(
            "https://www.wikidata.org/wiki/Special:EntityData/{}.json",
            wikidata_id
        );
    
        match gloo_net::http::Request::get(&url).send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    if let Some(entities) = data["entities"].as_object() {
                        if let Some(entity) = entities.get(wikidata_id) {
                            if let Some(claims) = entity["claims"].as_object() {
                                let mut result = HashMap::new();
                                for (property, values) in claims {
                                    if let Some(value) = values[0]["mainsnak"]["datavalue"]["value"].as_str() {
                                        result.insert(property.clone(), value.to_string());
                                    } else if let Some(value) = values[0]["mainsnak"]["datavalue"]["value"].as_object() {
                                        result.insert(property.clone(), serde_json::to_string(value).unwrap());
                                    } else if let Some(value) = values[0]["mainsnak"]["datavalue"]["value"].as_f64() {
                                        result.insert(property.clone(), value.to_string());
                                    } else {
                                        result.insert(property.clone(), "Unsupported data type".to_string());
                                    }
                                }
                                // Fetch labels for the properties
                                let property_ids = result.keys().cloned().collect::<Vec<_>>();
                                let labels = fetch_property_labels(property_ids).await;
                                set_property_labels.update(|labels_map| {
                                    for (key, value) in labels {
                                        labels_map.insert(key, value);
                                    }
                                });

                                // Update fetched properties
                                set_fetched_properties.update(|properties| {
                                    for (key, val) in result.clone() {
                                        properties.insert(key.clone(), val.clone());
                                    }
                                });
                                return result;
                            }
                        }
                    }
                }
            }
            Err(err) => log!("Error fetching item properties: {:?}", err),
        }
    
        HashMap::new()
    }
    
    async fn fetch_property_labels(property_ids: Vec<String>) -> HashMap<String, String> {
        let mut property_labels = HashMap::new();
    
        // Construct the API URL to fetch labels for multiple properties
        let url = format!(
            "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&props=labels&format=json&languages=en&origin=*",
            property_ids.join("|")
        );
    
        match gloo_net::http::Request::get(&url).send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    if let Some(entities) = data["entities"].as_object() {
                        for (property_id, entity) in entities {
                            if let Some(label) = entity["labels"]["en"]["value"].as_str() {
                                property_labels.insert(property_id.clone(), label.to_string());
                            }
                        }
                    }
                }
            }
            Err(err) => log!("Error fetching property labels: {:?}", err),
        }
    
        property_labels
    }
    
    // Add a new custom property
    let add_property = move |property: String| {
        set_custom_properties.update(|props| {
            if !props.contains(&property) && !property.is_empty() {
                props.push(property.clone());

                //update the selected_properties state when a new property is added
                set_selected_properties.update(|selected| {
                    selected.insert(property.clone(), true);
                });

                // Ensure the grid updates reactively
                set_items.update(|items| {
                    for item in items {
                        item.custom_properties.entry(property.clone()).or_insert_with(|| "".to_string());
                        
                        // Save the updated item to the database
                        let item_clone = item.clone();
                        spawn_local(async move {
                            save_item_to_db(item_clone, selected_properties).await;
                        });
                    }
                });

                // Fetch the property label
                let property_id = property.clone();
                spawn_local(async move {
                    let labels = fetch_property_labels(vec![property_id.clone()]).await;
                    set_property_labels.update(|labels_map| {
                        if let Some(label) = labels.get(&property_id) {
                            labels_map.insert(property_id, label.clone());
                        }
                    });
                });
            }
        });
        // Fetch the relevant value for each item and populate the corresponding cells
        set_items.update(|items| {
            for item in items {
                if let Some(wikidata_id) = &item.wikidata_id {
                    let wikidata_id = wikidata_id.clone();
                    let set_fetched_properties = set_fetched_properties.clone();
                    let set_property_labels = set_property_labels.clone();
                    let property_clone = property.clone();
                    spawn_local(async move {
                        let properties = fetch_item_properties(&wikidata_id, set_fetched_properties, set_property_labels).await;
                        // log!("Fetched properties for Wikidata ID {}: {:?}", wikidata_id, properties);
                        if let Some(value) = properties.get(&property_clone) {
                            set_items.update(|items| {
                                if let Some(item) = items.iter_mut().find(|item| item.wikidata_id.as_ref().unwrap() == &wikidata_id) {
                                    item.custom_properties.insert(property_clone.clone(), value.clone());
                                }
                            });
                        }
                    });
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
                        fetch_wikidata_suggestions(format!("name-{}", index), value.clone());

                        // Fetch Wikidata properties if the field is "name" and the item has a valid Wikidata ID
                        if !value.is_empty() {
                            if let Some(wikidata_id) = &item.wikidata_id {
                                let wikidata_id = wikidata_id.clone();
                                let set_fetched_properties = set_fetched_properties.clone();
                                let set_property_labels = set_property_labels.clone();
                                spawn_local(async move {
                                    let properties = fetch_item_properties(&wikidata_id, set_fetched_properties, set_property_labels).await;
                                    log!("Fetched properties for index {}: {:?}", index, properties);
                                });
                            }
                        }
                    }
                    "description" => {
                        item.description = value.clone();
                    }
                    _ => {
                        // Update custom property
                        item.custom_properties.insert(field.to_string(), value.clone());
                    }
                }

                // Save the updated item to the database
                let item_clone = item.clone();
                spawn_local(async move {
                    save_item_to_db(item_clone, selected_properties).await;
                });
            }
            // Automatically add a new row when editing the last row
            if index == items.len() - 1 && !value.is_empty() {
                let new_item = Item {
                    id: Uuid::new_v4().to_string(),
                    name: String::new(),
                    description: String::new(),
                    wikidata_id: None,
                    custom_properties: HashMap::new(),
                };
                items.push(new_item.clone());

                // Save the new item to the database
                spawn_local(async move {
                    save_item_to_db(new_item, selected_properties).await;
                });
            }
            log!("Items updated: {:?}", items);
        });
    };

    // List of properties to display as rows
    let properties = vec!["Name", "Description", "Actions"];

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
                        log!("Rendering property: {}", property);
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
                                                                set_show_suggestions.update(|suggestions| {
                                                                    suggestions.insert(format!("name-{}", index), true);
                                                                });
                                                            }))
                                                            on_blur=Some(Callback::new(move |_| {
                                                                log!("Input blurred, delaying hiding suggestions");
                                                                spawn_local(async move {
                                                                    gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
                                                                    log!("Hiding suggestions after delay");
                                                                    set_show_suggestions.update(|suggestions| {
                                                                        suggestions.insert(format!("name-{}", index), false);
                                                                    });
                                                                });
                                                            }))
                                                            input_type=InputType::Text
                                                        />
                                                        <button class="search-icon" on:click=move |_| {
                                                            log!("Search icon clicked, showing suggestions");
                                                            set_show_suggestions.update(|suggestions| {
                                                                suggestions.insert(format!("name-{}", index), true);
                                                            });
                                                        }> 
                                                            <i class="fas fa-search"></i> Search Wiki
                                                        </button>
                                                        {move || {
                                                            if *show_suggestions.get().get(&format!("name-{}", index)).unwrap_or(&false) {
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
                                                                                view! {
                                                                                    <li class="editable-cell-suggestions-li" on:click=move |_| {
                                                                                        // Update item with basic suggestion details
                                                                                        set_items.update(|items| {
                                                                                            if let Some(item) = items.get_mut(index) {
                                                                                                item.description = description_for_click.clone();
                                                                                                item.wikidata_id = Some(id.clone());
                                                                                                item.name = label_for_click.clone();
                                                                                            }
                                                                                        });

                                                                                        // Fetch additional properties from Wikidata
                                                                                        let wikidata_id = id.clone();
                                                                                        let set_fetched_properties = set_fetched_properties.clone();
                                                                                        let set_property_labels = set_property_labels.clone();
                                                                                        spawn_local(async move {
                                                                                            let properties = fetch_item_properties(&wikidata_id, set_fetched_properties, set_property_labels).await;
                                                                                            // log!("Fetched properties for Wikidata ID {}: {:?}", wikidata_id, properties);
                                                                                            
                                                                                            // Populate the custom properties for the new item
                                                                                            set_items.update(|items| {
                                                                                                if let Some(item) = items.iter_mut().find(|item| item.wikidata_id.as_ref() == Some(&wikidata_id)) {
                                                                                                    for (property, value) in properties {
                                                                                                        item.custom_properties.insert(property, value);
                                                                                                    }
                                                                                                }
                                                                                            });
                                                                                        });

                                                                                        // Hide the suggestion list
                                                                                        set_show_suggestions.update(|suggestions| {
                                                                                            suggestions.insert(format!("name-{}", index), false);
                                                                                            log!("Updated show_suggestions: {:?}", suggestions);
                                                                                        });
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
                                                    input_type=InputType::TextArea
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
                        log!("Rendering custom properties: {:?}", custom_props);
                        custom_props.into_iter().map(move |property| {
                            let property_clone = property.clone();
                            let property_label = property_labels.get().get(&property_clone).cloned().unwrap_or_else(|| property_clone.clone());
                            let property_clone_for_button = property_clone.clone();
                            let property_clone_for_cells = property_clone.clone(); 
                            view! {
                                <tr>
                                    <td>
                                        { property_label }
                                        <button class="delete-property" on:click=move |_| {
                                            log!("Deleting property: {}", property_clone_for_button);
                                            remove_property(property_clone_for_button.clone());
                                            set_custom_properties.update(|props| {
                                                props.retain(|p| p != &property_clone_for_button);
                                            });
                                            set_selected_properties.update(|selected| {
                                                selected.remove(&property_clone_for_button);
                                            });
                                            set_items.update(|items| {
                                                for item in items {
                                                    item.custom_properties.remove(&property_clone_for_button);
                                                }
                                            });
                                        }>{ "Delete" }</button>
                                    </td>
                                    {move || {
                                        let property_clone_for_cells = property_clone.clone();
                                        items.get().iter().enumerate().map(move |(index, item)| {
                                        let property_clone_for_closure = property_clone_for_cells.clone();
                                        view! {
                                            <td>
                                                <EditableCell
                                                    value=item.custom_properties.get(&property_clone_for_closure).cloned().unwrap_or_default()
                                                    on_input=move |value| update_item(index, &property_clone_for_closure, value)
                                                    key=Arc::new(format!("custom-{}-{}", property_clone_for_cells, index))
                                                    focused_cell=focused_cell
                                                    set_focused_cell=set_focused_cell.clone()
                                                    on_focus=Some(Callback::new(move |_| {
                                                        log!("Custom property input focused");
                                                    }))
                                                    on_blur=Some(Callback::new(move |_| {
                                                        log!("Custom property input blurred");
                                                    }))
                                                    input_type=InputType::TextArea
                                                />
                                            </td>
                                        }
                                    }).collect::<Vec<_>>()}
                                    }
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
            <div style="margin-bottom: 20px;">
            <input type="text" id="new-property" placeholder="Add New Property" list="properties"/>
                <datalist id="properties">
                    {move || {
                        let properties = fetched_properties.get().clone();
                        let property_labels = property_labels.get().clone();
                        properties.into_iter().map(|(key, _)| {
                            let key_clone = key.clone();
                            let label = property_labels.get(&key_clone).cloned().unwrap_or_else(|| key_clone.clone());
                            view! {
                                <option value={format!("{} - {}", key, label)}>{ format!("{} - {}", key, label) }</option>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </datalist>
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
                    // Extract the coded name from the selected value
                    let coded_name = property.split(" - ").next().unwrap_or(&property).to_string();
                                    
                    // Add the property using the coded name
                    add_property(coded_name);
                }>{ "Add Property" }</button>
            </div>
        </div>
    }
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    search: Vec<WikidataSuggestion>,
}