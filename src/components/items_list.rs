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
use web_sys::window;

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

//function to load items from database
pub async fn load_items_from_db(current_url: &str) -> Result<Vec<Item>, String> {
    let response = gloo_net::http::Request::get("/api/items?url={}")
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

        // log!("Deserialized DB items: {:?}", db_items);

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
                    custom_properties,
                }
            })
            .collect();
        // log!("Converted items: {:?}", items);
        Ok(items)
    } else {
        Err(format!("Failed to fetch items: {}", response.status_text()))
    }
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
    
    // State to manage suggestions visibility
    let (show_suggestions, set_show_suggestions) = create_signal(HashMap::<String, bool>::new());
    
    // cache to store fetched properties
    let (fetched_properties, set_fetched_properties) = create_signal(HashMap::<String, HashMap<String, String>>::new());
   
    // Signal to store the fetched property labels
    let (property_labels, set_property_labels) = create_signal(HashMap::<String, String>::new());
    
    fn get_current_url() -> String {
        window()
            .and_then(|win| win.location().href().ok())
            .unwrap_or_else(|| "".to_string())
    }

    let current_url = get_current_url();

    spawn_local(async move {
        match load_items_from_db(&current_url).await {
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

                // log!("Items after loading: {:?}", items.get());
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
                    // log!("Item saved to database: {:?}", item_to_send);
                } else {
                    log!("Failed to save item: {}", resp.status_text());
                }
            }
            Err(err) => log!("Failed to save item: {:?}", err),
        }
    }

    // Function to remove an item
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

    // Function to remove a property
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

    // State to store Wikidata suggestions
    let (wikidata_suggestions, set_wikidata_suggestions) = create_signal(HashMap::<String, Vec<WikidataSuggestion>>::new());

    // Function to fetch Wikidata suggestions
    let fetch_wikidata_suggestions = move |key: String, query: String| {
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
                        set_wikidata_suggestions.update(|suggestions| {
                            suggestions.insert(key, data.search);
                        });
                    }
                }
                Err(_) => log!("Failed to fetch Wikidata suggestions"),
            }
        });
    };

    //function to fetch properties
    async fn fetch_item_properties(wikidata_id: &str) -> HashMap<String, String> {
        let sparql_query = format!(
            r#"
            SELECT ?propLabel ?value ?valueLabel WHERE {{
              wd:{} ?prop ?statement.
              ?statement ?ps ?value.
              ?property wikibase:claim ?prop.
              ?property wikibase:statementProperty ?ps.
              SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
            }}
            "#,
            wikidata_id
        );
    
        let url = format!(
            "https://query.wikidata.org/sparql?query={}&format=json",
            urlencoding::encode(&sparql_query)
        );
    
        match gloo_net::http::Request::get(&url)
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    let mut result = HashMap::new();
                    if let Some(bindings) = data["results"]["bindings"].as_array() {
                        for binding in bindings {
                            let prop_label = binding["propLabel"]["value"].as_str().unwrap_or("").to_string();
                            let prop_label = prop_label.replace("http://www.wikidata.org/prop/", "");
                            let value_label = binding["valueLabel"]["value"].as_str().unwrap_or("").to_string();
                            result.insert(prop_label, value_label);
                            log!("result: {:?}", result);
                        }
                    }
                    result
                   
                } else {
                    HashMap::new()
                }
            }
            Err(_) => HashMap::new(),
        }
    }
    
    async fn fetch_property_labels(property_ids: Vec<String>) -> HashMap<String, String> {
        log!("Fetching property labels for properties: {:?}", property_ids);
        
        // Remove the "http://www.wikidata.org/prop/" prefix from property IDs
        let property_ids: Vec<String> = property_ids
            .into_iter()
            .map(|id| id.replace("http://www.wikidata.org/prop/", ""))
            .collect();
        
        let property_ids_str = property_ids.join(" wd:");
        let sparql_query = format!(
            r#"
            SELECT ?prop ?propLabel WHERE {{
              VALUES ?prop {{ wd:{} }}
              SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
            }}
            "#,
            property_ids_str
        );
    
        let url = format!(
            "https://query.wikidata.org/sparql?query={}&format=json",
            urlencoding::encode(&sparql_query)
        );
        log!("Sending request to URL: {}", url);
    
        match gloo_net::http::Request::get(&url)
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(response) => {
                log!("Received response from Wikidata. Status: {}", response.status());
                if response.status() != 200 {
                    log!("Error: Unexpected status code {}", response.status());
                    return HashMap::new();
                }
    
                match response.text().await {
                    Ok(text) => {
                        log!("Response body: {}", text);
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(data) => {
                                log!("Successfully parsed response from Wikidata");
                                let mut result = HashMap::new();
                                if let Some(bindings) = data["results"]["bindings"].as_array() {
                                    log!("Found {} bindings in response", bindings.len());
                                    for (i, binding) in bindings.iter().enumerate() {
                                        if let (Some(prop), Some(label)) = (
                                            binding["prop"]["value"].as_str(),
                                            binding["propLabel"]["value"].as_str()
                                        ) {
                                            let prop_id = prop.split('/').last().unwrap_or("").to_string();
                                            result.insert(prop_id.clone(), label.to_string());
                                            log!("Processed binding {}: prop_id = {}, label = {}", i, prop_id, label);
                                        } else {
                                            log!("Warning: Binding {} is missing prop or propLabel", i);
                                        }
                                    }
                                } else {
                                    log!("Warning: No bindings found in the response");
                                }
                                log!("Fetched {} property labels", result.len());
                                result
                            }
                            Err(e) => {
                                log!("Error parsing response from Wikidata: {:?}", e);
                                HashMap::new()
                            }
                        }
                    }
                    Err(e) => {
                        log!("Error reading response body: {:?}", e);
                        HashMap::new()
                    }
                }
            }
            Err(e) => {
                log!("Error fetching property labels from Wikidata: {:?}", e);
                HashMap::new()
            }
        }
    }
    
    // Add a new custom property
    let add_property = move |property: String| {
        // Normalize the property ID
        let normalized_property = property.replace("http://www.wikidata.org/prop/", "");
        
        set_custom_properties.update(|props| {
            if !props.contains(&normalized_property) && !normalized_property.is_empty() {
                props.push(normalized_property.clone());

                //update the selected_properties state when a new property is added
                set_selected_properties.update(|selected| {
                    selected.insert(normalized_property.clone(), true);
                });

                // Ensure the grid updates reactively
                set_items.update(|items| {
                    for item in items {
                        item.custom_properties.entry(normalized_property.clone()).or_insert_with(|| "".to_string());
                        
                        // Save the updated item to the database
                        let item_clone = item.clone();
                        spawn_local(async move {
                            save_item_to_db(item_clone, selected_properties).await;
                        });
                    }
                });

                // Fetch the property label
                let property_id = normalized_property.clone();
                spawn_local(async move {
                    let labels = fetch_property_labels(vec![property_id.clone()]).await;
                    log!("Fetched labels: {:?}", labels);
                    set_property_labels.update(|labels_map| {
                        for (key, value) in labels {
                            log!("Inserting label: {} -> {}", key, value);
                            labels_map.insert(key, value);
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
                        let properties = fetch_item_properties(&wikidata_id).await;
                        // Update fetched properties and property labels
                        set_fetched_properties.update(|fp| {
                            fp.insert(wikidata_id.clone(), properties.clone());
                        });
                        set_property_labels.update(|pl| {
                            for (key, value) in properties.iter() {
                                pl.entry(key.clone()).or_insert_with(|| value.clone());
                            }
                        });
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
                                    let properties = fetch_item_properties(&wikidata_id).await;
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
                    // reviews: vec![],
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
    let properties = vec!["Name", "Description"];

    view! {
        <div>
            <h1>{ "Items List" }</h1>
            <table>
                <thead>
                    <tr>
                        <th>{ "Property" }</th>
                        {move || items.get().iter().enumerate().map(|(index, item)| {
                            view! {
                                <th>
                                    { item.name.clone() }
                                    <button on:click=move |_| remove_item(index)>{ "Delete" }</button>
                                </th>
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
                                                                                            let properties = fetch_item_properties(&wikidata_id).await;
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
                            let normalized_property = property.replace("http://www.wikidata.org/prop/", "");
                            let property_label = property_labels.get().get(&normalized_property).cloned().unwrap_or_else(|| normalized_property.clone());
                            log!("Rendering property: {} -> {}", normalized_property, property_label);
                            let property_clone_for_button = normalized_property.clone();
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
                                        let property_clone_for_cells = normalized_property.clone();
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
                                                    }))
                                                    on_blur=Some(Callback::new(move |_| {
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
                <input type="text" id="new-property" placeholder="Add New Property" list="properties" on:keydown=move |event| {
                    if event.key() == "Enter"{
                        let input_element = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
                        let property = input_element.value();
                        if !property.is_empty() {
                            // Extract the coded name from the selected value
                            let coded_name = property.split(" - ").next().unwrap_or(&property).to_string();

                            // Add the property using the coded name
                            add_property(coded_name);

                            // Clear the input field
                            input_element.set_value("");
                        }
                    }
                } />
                <datalist id="properties">
                    {move || {
                        let property_labels = property_labels.get().clone();
                        property_labels.into_iter().map(|(property, label)| {
                            let property_clone = property.clone();
                            view! {
                                <option value={property}>{ format!("{} - {}", property_clone, label) }</option>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </datalist>
            </div>
        </div>
    }
}

#[derive(Deserialize, Clone, Debug)]
struct WikidataResponse {
    search: Vec<WikidataSuggestion>,
}