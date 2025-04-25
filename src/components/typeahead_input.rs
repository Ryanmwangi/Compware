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
use std::rc::Rc;
use std::cell::RefCell;

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
    
    // Create a unique ID for this component instance
    let component_id = format!("typeahead-{}", uuid::Uuid::new_v4());
    
    // Flag to track if component is mounted
    let is_mounted = Rc::new(RefCell::new(true));
    let is_mounted_clone = is_mounted.clone();
    
    // Clone necessary values for the async task
    let fetch_suggestions_clone = fetch_suggestions.clone();
    let on_select_clone = on_select.clone();
    let node_ref_clone = node_ref.clone();
    let component_id_clone = component_id.clone();
    
    // Create a cancellation token for the async task
    let (cancel_token, set_cancel_token) = create_signal(false);
    
    // Spawn the initialization task using spawn_local instead of spawn_local_with_handle
    spawn_local(async move {
        log!("[INIT] Component mounted: {}", component_id_clone);
        
        let mut retries = 0;
        while retries < 10 && !cancel_token.get() {
            // Check if component is still mounted before proceeding
            if !*is_mounted.borrow() {
                log!("[INIT] Component unmounted, aborting initialization: {}", component_id_clone);
                return;
            }
            
            if let Some(input) = node_ref_clone.get() {
                log!("[INIT] Input element found: {}", component_id_clone);
                
                // Only proceed if component is still mounted
                if !*is_mounted.borrow() || cancel_token.get() {
                    log!("[INIT] Component unmounted after input found, aborting: {}", component_id_clone);
                    return;
                }
                
                let bloodhound = initialize_bloodhound(fetch_suggestions_clone.clone(), &component_id_clone);
                
                // Store bloodhound in a component-specific global variable
                let bloodhound_var = format!("bloodhoundInstance_{}", component_id_clone.replace("-", "_"));
                if let Err(_) = js_sys::Reflect::set(
                    &js_sys::global(),
                    &bloodhound_var.into(),
                    &bloodhound
                ) {
                    log!("[ERROR] Failed to store bloodhound instance: {}", component_id_clone);
                }

                // Only proceed if component is still mounted
                if !*is_mounted.borrow() || cancel_token.get() {
                    log!("[INIT] Component unmounted before typeahead init, aborting: {}", component_id_clone);
                    return;
                }
                
                initialize_typeahead(&input, bloodhound, on_select_clone.clone(), node_ref_clone.clone(), &component_id_clone);
                
                // Only set initialized if component is still mounted
                if *is_mounted.borrow() && !cancel_token.get() {
                    // Use a try_update to safely update the signal
                    let _ = try_with_owner(Owner::current().unwrap(), move || {
                        set_initialized.set(true);
                    });
                }
                break;
            }
            
            // Check if component is still mounted before sleeping
            if !*is_mounted.borrow() || cancel_token.get() {
                log!("[INIT] Component unmounted during retry loop, aborting: {}", component_id_clone);
                return;
            }
            
            gloo_timers::future::sleep(Duration::from_millis(100)).await;
            retries += 1;
        }
    });
    
    // Clone component_id for on_cleanup
    let component_id_for_cleanup = component_id.clone();
    
    // Comprehensive cleanup function
    on_cleanup(move || {
        log!("[CLEANUP] TypeaheadInput component unmounting: {}", component_id_for_cleanup);
        
        // Signal the async task to cancel
        set_cancel_token.set(true);
        
        // Mark component as unmounted
        *is_mounted_clone.borrow_mut() = false;
        
        // Clean up component-specific global references
        let bloodhound_var = format!("bloodhoundInstance_{}", component_id_for_cleanup.replace("-", "_"));
        let prepare_fn_var = format!("bloodhoundPrepare_{}", component_id_for_cleanup.replace("-", "_"));
        
        let cleanup_script = format!(r#"
        try {{
            // Clean up the bloodhound instance
            if (window['{bloodhound_var}']) {{
                delete window['{bloodhound_var}'];
            }}
            
            // Clean up the prepare function
            if (window['{prepare_fn_var}']) {{
                delete window['{prepare_fn_var}'];
            }}
            
            // Clean up any typeahead instances
            if (window.typeaheadCleanupFunctions && window.typeaheadCleanupFunctions['{component_id}']) {{
                window.typeaheadCleanupFunctions['{component_id}']();
                delete window.typeaheadCleanupFunctions['{component_id}'];
            }}
            
            console.log('[JS] Global cleanup completed for {component_id}');
        }} catch (e) {{
            console.error('[JS] Cleanup error for {component_id}:', e);
        }}
        "#, 
            bloodhound_var = bloodhound_var,
            prepare_fn_var = prepare_fn_var,
            component_id = component_id_for_cleanup
        );
        
        if let Err(e) = js_sys::eval(&cleanup_script) {
            log!("[RUST] Cleanup script eval error: {:?}", e);
        }
    });

    // CSS
    let css = r#"
        .typeahead.tt-input {
            background: transparent !important;
        }

        .tt-menu {
            width: 100% !important;
            background: white;
            border: 1px solid #ddd;
            border-radius: 4px;
            box-shadow: 0 5px 10px rgba(0,0,0,.2);
            max-height: 300px;
            overflow-y: auto;
            z-index: 1000 !important;
        }

        .tt-dataset-suggestions {
            padding: 8px 0;
        }

        .suggestion-item * {
            pointer-events: none;  /* Prevent element interception */
            white-space: nowrap;   /* Prevent text wrapping */
            overflow: hidden;      /* Hide overflow */
            text-overflow: ellipsis; /* Add ellipsis for long text */
        }

        .suggestion-item {
            padding: 8px 15px;
            border-bottom: 1px solid #eee;
        }

        .suggestion-item:hover {
            background-color: #f8f9fa;
            cursor: pointer;
        }

        .label {
            font-weight: 500;
            color: #333;
        }

        .description {
            font-size: 0.9em;
            color: #666;
            margin-top: 2px;
        }

        .empty-suggestion {
            padding: 8px 15px;
            color: #999;
        }
    "#;

    // Clone component_id for event handlers
    let component_id_for_focus = component_id.clone();
    let component_id_for_blur = component_id.clone();
    let component_id_for_input = component_id.clone();

    view! {
        <style>
            {css}
        </style>

        <input
            type="text"
            class="typeahead-input"
            prop:value=value
            node_ref=node_ref
            on:focus=move |_| log!("[FOCUS] Name input focused: {}", component_id_for_focus)
            on:blur=move |_| log!("[FOCUS] Name input blurred: {}", component_id_for_blur)
            on:input=move |ev| {
                let value = event_target_value(&ev);
                log!("[INPUT] Value changed: {} ({})", value, component_id_for_input);
                
                // If this is the last row and we have an on_input callback, call it
                if is_last_row && !value.is_empty() {
                    if let Some(callback) = &on_input {
                        callback.call(value.clone());
                    }
                }
            }
        />
    }
}

fn initialize_bloodhound(fetch: Callback<String, Vec<WikidataSuggestion>>, component_id: &str) -> JsValue {
    let bloodhound_options = Object::new();
    
    // Use component-specific names for global functions
    let prepare_fn_var = format!("bloodhoundPrepare_{}", component_id.replace("-", "_"));
    
    // Create a closure that will be called by Bloodhound to fetch suggestions
    let remote_fn = Closure::wrap(Box::new(move |query: JsValue, sync: Function, _async_fn: Function| {
        let query_str = query.as_string().unwrap_or_default();
        log!("[BLOODHOUND] Fetching suggestions for: {}", query_str);
        
        // Safely call the fetch callback
        let suggestions = match try_with_owner(Owner::current().unwrap(), move || {
            fetch.call(query_str.clone())
        }) {
            Ok(suggs) => suggs,
            Err(e) => {
                log!("[ERROR] Failed to fetch suggestions: {:?}", e);
                Vec::new()
            }
        };
        
        // Create a JavaScript array to hold the suggestions
        let js_suggestions = Array::new();
        
        // Convert each suggestion to a JavaScript object
        for suggestion in suggestions {
            let obj = Object::new();
            
            // Store the original ID, label, and description
            Reflect::set(&obj, &"id".into(), &JsValue::from_str(&suggestion.id)).unwrap_or_default();
            Reflect::set(&obj, &"label".into(), &JsValue::from_str(&suggestion.label)).unwrap_or_default();
            Reflect::set(&obj, &"description".into(), &JsValue::from_str(&suggestion.description)).unwrap_or_default();
            
            // Store the display values directly on the object for easier access
            Reflect::set(&obj, &"displayLabel".into(), 
                &JsValue::from_str(&suggestion.display.label.value)).unwrap_or_default();
            Reflect::set(&obj, &"displayDescription".into(), 
                &JsValue::from_str(&suggestion.display.description.value)).unwrap_or_default();
            
            // Store the full suggestion for later retrieval
            if let Ok(full_suggestion) = JsValue::from_serde(&suggestion) {
                Reflect::set(&obj, &"fullSuggestion".into(), &full_suggestion).unwrap_or_default();
            }
            
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
        &format!(r#"
        // Call our custom prepare function directly
        if (window['{prepare_fn_var}']) {{
            window['{prepare_fn_var}'](query, syncResults, asyncResults);
        }} else {{
            console.error('[JS] Prepare function not found: {prepare_fn_var}');
            syncResults([]);
        }}
        "#, prepare_fn_var = prepare_fn_var)
    );
    
    Reflect::set(
        &remote_config,
        &"transport".into(),
        &transport_fn
    ).unwrap_or_default();
    
    // Set a dummy URL (not actually used with custom transport)
    Reflect::set(
        &remote_config,
        &"url".into(),
        &JsValue::from_str("/dummy?query=%QUERY")
    ).unwrap_or_default();
    
    // Store our prepare function globally with component-specific name
    // Clone prepare_fn_var before using .into() which consumes it
    let prepare_fn_var_for_log = prepare_fn_var.clone();
    if let Err(_) = js_sys::Reflect::set(
        &js_sys::global(),
        &prepare_fn_var.into(),
        remote_fn.as_ref().unchecked_ref()
    ) {
        log!("[ERROR] Failed to store prepare function: {}", prepare_fn_var_for_log);
    }
    
    // Set rate limiting to prevent too many requests
    Reflect::set(
        &remote_config,
        &"rateLimitWait".into(),
        &JsValue::from(300)
    ).unwrap_or_default();
    
    // Set the wildcard for query replacement
    Reflect::set(
        &remote_config,
        &"wildcard".into(),
        &JsValue::from_str("%QUERY")
    ).unwrap_or_default();
    
    // Add the remote config to the options
    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).unwrap_or_default();
    
    // Set the tokenizers
    let tokenizer = js_sys::Function::new_no_args(
        r#"
        return function(datum) {
            return datum.toString().trim().split(/\s+/);
        };
        "#
    );
    
    Reflect::set(
        &bloodhound_options, 
        &"datumTokenizer".into(), 
        &tokenizer
    ).unwrap_or_default();
    
    Reflect::set(
        &bloodhound_options, 
        &"queryTokenizer".into(), 
        &tokenizer
    ).unwrap_or_default();
    
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
    component_id: &str,
) {
    log!("[TYPEAHEAD] Initializing for input: {} ({})", input.id(), component_id);
    let input_id = format!("typeahead-input-{}", component_id);
    input.set_id(&input_id);

    // Create selection handler closure
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event, suggestion: JsValue| {
        log!("[TYPEAHEAD] Selection made");
        
        // Safely call the on_select callback
        let _ = try_with_owner(Owner::current().unwrap(), move || {
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
        });
    }) as Box<dyn FnMut(web_sys::Event, JsValue)>);

    // Register global handler with component-specific name
    let handler_name = format!("handler_{}", input_id.replace("-", "_"));
    
    log!("[TYPEAHEAD] Registering handler with name: {}", handler_name);
    
    // Clone handler_name before using it in error log
    let handler_name_for_log = handler_name.clone();
    if let Err(_) = js_sys::Reflect::set(
        &js_sys::global(),
        &handler_name.clone().into(),
        closure.as_ref().unchecked_ref(),
    ) {
        log!("[ERROR] Failed to register handler: {}", handler_name_for_log);
    }
    
    // We'll clean this up in the component's cleanup function
    closure.forget();

    // Register cleanup function for this specific typeahead instance
    let cleanup_script = format!(
        r#"
        // Store a reference to the cleanup function for this input
        if (!window.typeaheadCleanupFunctions) {{
            window.typeaheadCleanupFunctions = {{}};
        }}
        
        window.typeaheadCleanupFunctions['{component_id}'] = function() {{
            try {{
                // Destroy the typeahead instance
                $('#{input_id}').typeahead('destroy');
                
                // Remove the handler
                if (window['{handler_name}']) {{
                    delete window['{handler_name}'];
                }}
                
                console.log('[JS] Typeahead cleanup for #{input_id} completed');
            }} catch (e) {{
                console.error('[JS] Typeahead cleanup error for #{input_id}:', e);
            }}
        }};
        "#,
        component_id = component_id,
        input_id = input_id,
        handler_name = handler_name_for_log
    );
    
    if let Err(e) = js_sys::eval(&cleanup_script) {
        log!("[RUST] Cleanup script eval error: {:?}", e);
    }

    // Get the bloodhound instance from the component-specific global variable
    let bloodhound_var = format!("bloodhoundInstance_{}", component_id.replace("-", "_"));
    
    // Initialization script with enhanced logging and error handling
    let init_script = format!(
        r#"
        console.log('[JS] Starting Typeahead init for #{input_id}');
        try {{
            // Get the bloodhound instance from our component-specific variable
            var bloodhound = window['{bloodhound_var}'];
            if (!bloodhound) {{
                throw new Error('Bloodhound instance not found: {bloodhound_var}');
            }}
            
            // Define a custom source function that directly uses our Rust callback
            var customSource = function(query, syncResults, asyncResults) {{
                console.log('[JS] Custom source called with query:', query);
                
                // Get the prepare function from our component-specific variable
                var prepareFn = window['{prepare_fn_var}'];
                if (!prepareFn) {{
                    console.error('[JS] Prepare function not found: {prepare_fn_var}');
                    syncResults([]);
                    return;
                }}
                
                // Call our prepare function
                prepareFn(query, function(suggestions) {{
                    console.log('[JS] Suggestions from custom source:', suggestions);
                    syncResults(suggestions);
                }}, asyncResults);
            }};
            
            // Initialize typeahead with error handling
            var $input = $('#{input_id}');
            if ($input.length === 0) {{
                throw new Error('Input element not found: #{input_id}');
            }}
            
            $input.typeahead(
                {{
                    hint: true,
                    highlight: true,
                    minLength: 1
                }},
                {{
                    name: 'suggestions',
                    display: function(data) {{
                        if (!data) return '';
                        return data.displayLabel || data.label || '';
                    }},
                    source: customSource,
                    templates: {{
                        suggestion: function(data) {{
                            if (!data) return $('<div>').addClass('empty-suggestion').text('Invalid data');
                            
                            return $('<div>')
                                .addClass('suggestion-item')
                                .append($('<div>').addClass('label').text(data.displayLabel || data.label || ''))
                                .append($('<div>').addClass('description').text(data.displayDescription || data.description || ''));
                        }},
                        empty: function() {{
                            return $('<div>').addClass('empty-suggestion').text('No matches found');
                        }}
                    }}
                }}
            )
            .on('typeahead:select', function(ev, suggestion) {{
                if (!suggestion) {{
                    console.error('[JS] Selection event received with null suggestion');
                    return;
                }}
                
                console.log('[JS] Selection event received with suggestion:', suggestion);
                
                // Get the handler from our component-specific variable
                var handler = window['{handler_name}'];
                if (!handler) {{
                    console.error('[JS] Handler function not found: {handler_name}');
                    return;
                }}
                
                // Call the handler
                handler(ev, suggestion);
            }});
            
            console.log('[JS] Typeahead initialized successfully for #{input_id}');
        }} catch (e) {{
            console.error('[JS] Typeahead init error for #{input_id}:', e);
        }}
        "#,
        input_id = input_id,
        bloodhound_var = bloodhound_var,
        prepare_fn_var = format!("bloodhoundPrepare_{}", component_id.replace("-", "_")),
        handler_name = handler_name_for_log
    );

    log!("[RUST] Running initialization script for: {}", input_id);
    if let Err(e) = js_sys::eval(&init_script) {
        log!("[RUST] Eval error: {:?}", e);
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