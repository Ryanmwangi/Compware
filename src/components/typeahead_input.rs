use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::item::WikidataSuggestion;
use js_sys::{Object, Array, Function, Reflect};
use leptos::html::Input; 
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement; 
use leptos::logging::log;
use std::time::Duration;

#[component]
pub fn TypeaheadInput(
    value: String,
    on_select: Callback<WikidataSuggestion>,
    fetch_suggestions: Callback<String, Vec<WikidataSuggestion>>,
    node_ref: NodeRef<Input>,
    #[prop(optional)] is_last_row: bool,
    #[prop(optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let (is_initialized, set_initialized) = create_signal(false);
    
    spawn_local(async move {
        log!("[INIT] Component mounted");
        
        let mut retries = 0;
        while retries < 10 {
            if let Some(input) = node_ref.get() {
                log!("[INIT] Input element found");
                let bloodhound = initialize_bloodhound(fetch_suggestions.clone());
                
                // Store bloodhound globally
                js_sys::Reflect::set(
                    &js_sys::global(),
                    &"bloodhoundInstance".into(),
                    &bloodhound
                ).unwrap();

                initialize_typeahead(&input, bloodhound, on_select.clone(), node_ref.clone());
                set_initialized.set(true);
                break;
            }
            gloo_timers::future::sleep(Duration::from_millis(100)).await;
            retries += 1;
        }
    });

    view! {
        <style>
            {r#"
            .typeahead.tt-input {{
                background: transparent !important;
            }}

            .tt-menu {{
                width: 100% !important;
                background: white;
                border: 1px solid #ddd;
                border-radius: 4px;
                box-shadow: 0 5px 10px rgba(0,0,0,.2);
                max-height: 300px;
                overflow-y: auto;
                z-index: 1000 !important;
            }}

            .tt-dataset-suggestions {{
                padding: 8px 0;
            }}

            .suggestion-item * {{
                pointer-events: none;  /* Prevent element interception */
                white-space: nowrap;   /* Prevent text wrapping */
                overflow: hidden;      /* Hide overflow */
                text-overflow: ellipsis; /* Add ellipsis for long text */
            }}

            .suggestion-item {{
                padding: 8px 15px;
                border-bottom: 1px solid #eee;
            }}

            .suggestion-item:hover {{
                background-color: #f8f9fa;
                cursor: pointer;
            }}

            .label {{
                font-weight: 500;
                color: #333;
            }}

            .description {{
                font-size: 0.9em;
                color: #666;
                margin-top: 2px;
            }}

            .empty-suggestion {{
                padding: 8px 15px;
                color: #999;
            }}
        "#}
        </style>

        <input
            type="text"
            class="typeahead-input"
            prop:value=value
            node_ref=node_ref
            on:focus=move |_| log!("[FOCUS] Name input focused")
            on:blur=move |_| log!("[FOCUS] Name input blurred")
            on:input=move |ev| {
                let value = event_target_value(&ev);
                log!("[INPUT] Value changed: {}", value);
                
                // If this is the last row and we have an on_input callback, call it
                if is_last_row && !value.is_empty() {
                    if let Some(callback) = &on_input {
                        callback.call(value.clone());
                    }
                }
                
                let _ = js_sys::eval("console.log('jQuery version:', $.fn.jquery)");
                let _ = js_sys::eval("console.log('Typeahead version:', $.fn.typeahead ? 'loaded' : 'missing')");
            }
        />
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "Bloodhound")]
    type Bloodhound;

    #[wasm_bindgen(constructor, js_namespace = window)]
    fn new(options: &JsValue) -> Bloodhound;

    #[wasm_bindgen(method)]
    fn initialize(this: &Bloodhound, prefetch: bool);

    #[wasm_bindgen(method, js_name = "ttAdapter")]
    fn tt_adapter(this: &Bloodhound) -> JsValue;
}


fn initialize_bloodhound(fetch: Callback<String, Vec<WikidataSuggestion>>) -> JsValue {
    let bloodhound_options = Object::new();
    
    // Create a closure that will be called by Bloodhound to fetch suggestions
    let remote_fn = Closure::wrap(Box::new(move |query: JsValue, sync: Function, async_fn: Function| {
        let query_str = query.as_string().unwrap_or_default();
        log!("[BLOODHOUND] Fetching suggestions for: {}", query_str);
        
        // Get suggestions from the callback
        let suggestions = fetch.call(query_str.clone());
        
        // Create a JavaScript array to hold the suggestions
        let js_suggestions = Array::new();
        
        // Convert each suggestion to a JavaScript object
        for suggestion in suggestions {
            let obj = Object::new();
            
            // Store the original ID, label, and description
            Reflect::set(&obj, &"id".into(), &JsValue::from_str(&suggestion.id)).unwrap();
            Reflect::set(&obj, &"label".into(), &JsValue::from_str(&suggestion.label)).unwrap();
            Reflect::set(&obj, &"description".into(), &JsValue::from_str(&suggestion.description)).unwrap();
            
            // Store the display values directly on the object for easier access
            Reflect::set(&obj, &"displayLabel".into(), 
                &JsValue::from_str(&suggestion.display.label.value)).unwrap();
            Reflect::set(&obj, &"displayDescription".into(), 
                &JsValue::from_str(&suggestion.display.description.value)).unwrap();
            
            // Store the full suggestion for later retrieval
            let full_suggestion = JsValue::from_serde(&suggestion).unwrap();
            Reflect::set(&obj, &"fullSuggestion".into(), &full_suggestion).unwrap();
            
            // Add the object to the array
            js_suggestions.push(&obj);
        }
        
        log!("[BLOODHOUND] Processed suggestions: {:?}", js_suggestions);
        
        // Call the sync function with the suggestions
        let _ = sync.call1(&JsValue::NULL, &js_suggestions);
    }) as Box<dyn Fn(JsValue, Function, Function)>);

    // Configure the remote options
    let remote_config = Object::new();
    
    // Set transport function to avoid AJAX requests
    let transport_fn = js_sys::Function::new_with_args(
        "query, syncResults, asyncResults",
        r#"
        // Call our custom prepare function directly
        window.bloodhoundPrepare(query, syncResults, asyncResults);
        "#
    );
    
    Reflect::set(
        &remote_config,
        &"transport".into(),
        &transport_fn
    ).unwrap();
    
    // Set a dummy URL (not actually used with custom transport)
    Reflect::set(
        &remote_config,
        &"url".into(),
        &JsValue::from_str("/dummy?query=%QUERY")
    ).unwrap();
    
    // Store our prepare function globally
    js_sys::Reflect::set(
        &js_sys::global(),
        &"bloodhoundPrepare".into(),
        remote_fn.as_ref().unchecked_ref()
    ).unwrap();
    
    // Set rate limiting to prevent too many requests
    Reflect::set(
        &remote_config,
        &"rateLimitWait".into(),
        &JsValue::from(300)
    ).unwrap();
    
    // Set the wildcard for query replacement
    Reflect::set(
        &remote_config,
        &"wildcard".into(),
        &JsValue::from_str("%QUERY")
    ).unwrap();
    
    // Add the remote config to the options
    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).unwrap();
    
    // Set the tokenizers
    let tokenizer = js_sys::Function::new_no_args(
        r#"
        return function(query) {
            return query.trim().split(/\s+/);
        }
        "#
    );
    
    Reflect::set(
        &bloodhound_options, 
        &"datumTokenizer".into(), 
        &tokenizer
    ).unwrap();
    
    Reflect::set(
        &bloodhound_options, 
        &"queryTokenizer".into(), 
        &tokenizer
    ).unwrap();
    
    // Create and initialize the Bloodhound instance
    let bloodhound = Bloodhound::new(&bloodhound_options.into());
    bloodhound.initialize(true);
    
    // Prevent the closure from being garbage collected
    remote_fn.forget();
    
    // Return the Bloodhound instance
    bloodhound.into()
}

fn initialize_typeahead(
    input: &HtmlInputElement,
    bloodhound: JsValue,
    on_select: Callback<WikidataSuggestion>,
    node_ref: NodeRef<Input>,
) {
    log!("[TYPEAHEAD] Initializing for input: {}", input.id());
    let input_id = format!("typeahead-{}", uuid::Uuid::new_v4());
    input.set_id(&input_id);

    // Create selection handler closure
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event, suggestion: JsValue| {
        log!("[TYPEAHEAD] Selection made");
        
        // Try to get the full suggestion from the suggestion object
        if let Some(full_suggestion) = js_sys::Reflect::get(&suggestion, &"fullSuggestion".into()).ok() {
            if let Ok(data) = full_suggestion.into_serde::<WikidataSuggestion>() {
                log!("[TYPEAHEAD] Selected suggestion: {:?}", data);
                on_select.call(data.clone());
                if let Some(input) = node_ref.get() {
                    input.set_value(&data.label);
                }
                return;
            }
        }
        
        // Fallback: try to deserialize the suggestion directly
        if let Ok(data) = suggestion.into_serde::<WikidataSuggestion>() {
            log!("[TYPEAHEAD] Selected suggestion (fallback): {:?}", data);
            on_select.call(data.clone());
            if let Some(input) = node_ref.get() {
                input.set_value(&data.label);
            }
        } else {
            log!("[ERROR] Failed to deserialize suggestion");
        }
    }) as Box<dyn FnMut(web_sys::Event, JsValue)>);

    // Register global handler 
    let handler_name = format!("handler_{}", input_id.replace("-", "_"));
    
    log!("[TYPEAHEAD] Registering handler with name: {}", handler_name);
    
    js_sys::Reflect::set(
        &js_sys::global(),
        &handler_name.clone().into(),
        closure.as_ref().unchecked_ref(),
    ).unwrap();
    closure.forget();

    // Initialization script with enhanced logging
    let init_script = format!(
        r#"
        console.log('[JS] Starting Typeahead init for #{id}');
        try {{
            var bloodhound = window.bloodhoundInstance;
            
            // Define a custom source function that directly uses our Rust callback
            var customSource = function(query, syncResults, asyncResults) {{
                console.log('[JS] Custom source called with query:', query);
                
                // Call our global prepare function directly
                window.bloodhoundPrepare(query, function(suggestions) {{
                    console.log('[JS] Suggestions from custom source:', suggestions);
                    syncResults(suggestions);
                }}, asyncResults);
            }};
            
            $('#{id}').typeahead(
                {{
                    hint: true,
                    highlight: true,
                    minLength: 1
                }},
                {{
                    name: 'suggestions',
                    display: function(data) {{
                        console.log('[JS] Display function called with data:', data);
                        return data.displayLabel || data.label || '';
                    }},
                    source: customSource,
                    templates: {{
                        suggestion: function(data) {{
                            console.log('[JS] Rendering suggestion:', data);
                            return $('<div>')
                                .addClass('suggestion-item')
                                .append($('<div>').addClass('label').text(data.displayLabel || data.label))
                                .append($('<div>').addClass('description').text(data.displayDescription || data.description));
                        }},
                        empty: function() {{
                            console.log('[JS] No suggestions found');
                            return $('<div>').addClass('empty-suggestion').text('No matches found');
                        }}
                    }}
                }}
            )
            .on('typeahead:select', function(ev, suggestion) {{
                console.log('[JS] Selection event received with suggestion:', suggestion);
                window['{handler}'](ev, suggestion);
            }});
            console.log('[JS] Typeahead initialized successfully');
        }} catch (e) {{
            console.error('[JS] Typeahead init error:', e);
        }}
        "#,
        id = input_id,
        handler = handler_name
    );

    log!("[RUST] Initialization script: {}", init_script);
    if let Err(e) = js_sys::eval(&init_script) {
        log!("[RUST] Eval error: {:?}", e);
    }
}