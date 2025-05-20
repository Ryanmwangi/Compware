use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::item::WikidataSuggestion;
use leptos::html::Input;
use leptos::logging::log;
use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;

// Only include these imports when targeting wasm
#[cfg(target_arch = "wasm32")]
use {
    js_sys::{Object, Array, Function, Reflect},
    gloo_utils::format::JsValueSerdeExt,
    wasm_bindgen::JsCast,
    web_sys::HtmlInputElement,
    std::sync::atomic::{AtomicBool, Ordering},
    std::sync::Arc,
};

// Add Bloodhound wrapper struct - only for wasm
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window"])]
    type Bloodhound;

    #[wasm_bindgen(constructor, js_namespace = ["window"])]
    fn new(options: &JsValue) -> Bloodhound;

    #[wasm_bindgen(method, js_namespace = ["window"])]
    fn initialize(this: &Bloodhound, reinitialize: bool);
}

// Enhanced closure management with explicit lifecycle tracking - only for wasm
#[cfg(target_arch = "wasm32")]
struct TypeaheadClosures {
    selection_closure: Option<Closure<dyn FnMut(web_sys::Event, JsValue)>>,
    remote_fn_closure: Option<Closure<dyn FnMut(JsValue, Function, Function)>>,
    // Track if component is still alive to prevent accessing invalid memory
    is_alive: Arc<AtomicBool>,
}

#[cfg(target_arch = "wasm32")]
impl TypeaheadClosures {
    fn new() -> Self {
        Self {
            selection_closure: None,
            remote_fn_closure: None,
            is_alive: Arc::new(AtomicBool::new(true)),
        }
    }

    fn cleanup(&mut self) {
        // Mark as no longer alive before dropping closures
        self.is_alive.store(false, Ordering::SeqCst);
        
        // Take ownership of closures to drop them
        let _ = self.selection_closure.take();
        let _ = self.remote_fn_closure.take();
        
        log!("[CLEANUP] TypeaheadClosures cleaned up");
    }
    
    // Get a clone of the is_alive flag for use in closures
    fn get_alive_flag(&self) -> Arc<AtomicBool> {
        self.is_alive.clone()
    }
}

// Drop implementation to ensure cleanup happens - only for wasm
#[cfg(target_arch = "wasm32")]
impl Drop for TypeaheadClosures {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// Create a dummy TypeaheadClosures for non-wasm targets
#[cfg(not(target_arch = "wasm32"))]
struct TypeaheadClosures {}

#[cfg(not(target_arch = "wasm32"))]
impl TypeaheadClosures {
    fn new() -> Self {
        Self {}
    }
    
    fn cleanup(&mut self) {
        // No-op for non-wasm
    }
}

#[component]
pub fn TypeaheadInput(
    value: String,
    on_select: Callback<WikidataSuggestion>,
    fetch_suggestions: Callback<String, Vec<WikidataSuggestion>>,
    node_ref: NodeRef<Input>,
    #[prop(optional)] is_last_row: bool,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] should_focus: bool,
) -> impl IntoView {
    let (is_initialized, set_initialized) = create_signal(false);
    
    // Create a unique ID for this component instance
    let component_id = format!("typeahead-{}", uuid::Uuid::new_v4());
    
    // Clone component_id before moving it into the closure
    let component_id_for_effect = component_id.clone();
    // Effect to handle focus when should_focus is true
    create_effect(move |_| {
        if should_focus {
            if let Some(input) = node_ref.get() {
                let _ = input.focus();
                log!("[FOCUS] Auto-focusing input: {}", component_id_for_effect);
            }
        }
    });

    // WASM-specific initialization
    #[cfg(target_arch = "wasm32")]
    {
        // Create a storage for closures with explicit lifecycle tracking
        let closures = Rc::new(RefCell::new(TypeaheadClosures::new()));
        let closures_for_cleanup = closures.clone();
        
        // Clone necessary values for the async task
        let fetch_suggestions_clone = fetch_suggestions.clone();
        let on_select_clone = on_select.clone();
        let node_ref_clone = node_ref.clone();
        let component_id_clone = component_id.clone();
        let closures_clone = closures.clone();
        
        // Create a cancellation token for the async task
        let (cancel_token, set_cancel_token) = create_signal(false);
        
        // Register global cleanup function in JavaScript
        let register_cleanup_script = format!(
            r#"
            // Create a global registry for typeahead components if it doesn't exist
            if (!window.typeaheadRegistry) {{
                window.typeaheadRegistry = {{}};
            }}
            
            // Register this component
            window.typeaheadRegistry['{component_id}'] = {{
                initialized: false,
                bloodhound: null,
                handlers: {{}},
                // EXPLICIT ALIVE FLAG
                alive: true,
                // Method to safely call handlers
                callHandler: function(handlerName, ...args) {{
                    // DEFENSIVE: Check alive flag before calling any handler
                    if (!this.alive) {{
                        console.warn('[JS] Component {component_id} is no longer alive, ignoring handler call: ' + handlerName);
                        return null;
                    }}
                    
                    try {{
                        const handler = this.handlers[handlerName];
                        if (handler && typeof handler === 'function') {{
                            return handler(...args);
                        }}
                    }} catch (e) {{
                        console.error('[JS] Error calling handler:', e);
                    }}
                    return null;
                }},
                // Method to register a handler
                registerHandler: function(name, fn) {{
                    // DEFENSIVE: Don't register handlers if component is not alive
                    if (!this.alive) {{
                        console.warn('[JS] Component {component_id} is no longer alive, ignoring handler registration: ' + name);
                        return false;
                    }}
                    
                    this.handlers[name] = fn;
                    return true;
                }},
                // Method to unregister a handler
                unregisterHandler: function(name) {{
                    if (this.handlers[name]) {{
                        delete this.handlers[name];
                        return true;
                    }}
                    return false;
                }},
                // Method to clean up all resources
                cleanup: function() {{
                    try {{
                        // IMPORTANT: Set alive to false FIRST to prevent any new calls
                        this.alive = false;
                        console.log('[JS] Component {component_id} marked as not alive');
                        
                        // Clean up typeahead
                        const inputId = 'typeahead-input-{component_id}';
                        const $input = $('#' + inputId);
                        if ($input.length > 0) {{
                            // Remove all event handlers first
                            $input.off('typeahead:select');
                            $input.off('typeahead:active');
                            $input.off('typeahead:idle');
                            $input.off('typeahead:open');
                            $input.off('typeahead:close');
                            $input.off('typeahead:change');
                            $input.off('typeahead:render');
                            $input.off('typeahead:autocomplete');
                            $input.off('typeahead:cursorchange');
                            $input.off('typeahead:asyncrequest');
                            $input.off('typeahead:asynccancel');
                            $input.off('typeahead:asyncreceive');
                            console.log('[JS] Removed all typeahead event handlers for {component_id}');
                            
                            // Now destroy the typeahead
                            $input.typeahead('destroy');
                            console.log('[JS] Destroyed typeahead for {component_id}');
                        }}
                        
                        // Explicitly null out the global handler references
                        if (window.rustSelectHandler_{component_id_safe}) {{
                            window.rustSelectHandler_{component_id_safe} = null;
                            console.log('[JS] Nulled rustSelectHandler_{component_id_safe}');
                        }}
                        
                        if (window.rustFetchHandler_{component_id_safe}) {{
                            window.rustFetchHandler_{component_id_safe} = null;
                            console.log('[JS] Nulled rustFetchHandler_{component_id_safe}');
                        }}
                        
                        // Clear all handlers
                        this.handlers = {{}};
                        
                        // Mark as cleaned up
                        this.initialized = false;
                        this.bloodhound = null;
                        
                        console.log('[JS] Component {component_id} cleaned up successfully');
                        return true;
                    }} catch (e) {{
                        console.error('[JS] Error during cleanup:', e);
                        // Still mark as not alive even if cleanup fails
                        this.alive = false;
                        return false;
                    }}
                }}
            }};
            
            console.log('[JS] Registered component {component_id}');
            true
            "#,
            component_id = component_id,
            component_id_safe = component_id.replace("-", "_")
        );
        
        // Execute the registration script
        match js_sys::eval(&register_cleanup_script) {
            Ok(_) => log!("[INIT] Registered cleanup handlers for {}", component_id),
            Err(e) => log!("[ERROR] Failed to register cleanup handlers: {:?}", e),
        }
        
        // Spawn the initialization task
        spawn_local(async move {
            log!("[INIT] Component mounted: {}", component_id_clone);
            
            let mut retries = 0;
            while retries < 10 && !cancel_token.get() {
                // Check if component is still alive
                if !closures_clone.borrow().is_alive.load(Ordering::SeqCst) {
                    log!("[INIT] Component no longer alive, aborting initialization: {}", component_id_clone);
                    return;
                }
                
                if let Some(input) = node_ref_clone.get() {
                    log!("[INIT] Input element found: {}", component_id_clone);
                    
                    // Check again if component is still alive
                    if !closures_clone.borrow().is_alive.load(Ordering::SeqCst) || cancel_token.get() {
                        log!("[INIT] Component no longer alive after input found, aborting: {}", component_id_clone);
                        return;
                    }
                    
                    // Initialize Bloodhound with safe closure handling
                    let bloodhound = match initialize_bloodhound(fetch_suggestions_clone.clone(), &component_id_clone, closures_clone.clone()) {
                        Ok(bh) => bh,
                        Err(e) => {
                            log!("[ERROR] Failed to initialize Bloodhound: {:?}", e);
                            return;
                        }
                    };
                    
                    // Store bloodhound in the component registry
                    let store_bloodhound_script = format!(
                        r#"
                        if (window.typeaheadRegistry && window.typeaheadRegistry['{component_id}']) {{
                            window.typeaheadRegistry['{component_id}'].bloodhound = bloodhoundInstance;
                            console.log('[JS] Stored bloodhound instance for {component_id}');
                            true
                        }} else {{
                            console.error('[JS] Component registry not found for {component_id}');
                            false
                        }}
                        "#,
                        component_id = component_id_clone
                    );
                    
                    // Convert bloodhound to a JSON string
                    let bloodhound_json = match js_sys::JSON::stringify(&bloodhound) {
                        Ok(json_str) => json_str.as_string().unwrap_or_default(),
                        Err(_) => "{}".to_string()
                    };
                    
                    // Create the full script
                    let full_script = format!(
                        "var bloodhoundInstance = {}; {}",
                        bloodhound_json,
                        store_bloodhound_script
                    );
                    
                    // Evaluate the script
                    let bloodhound_stored = match js_sys::eval(&full_script) {
                        Ok(result) => result.as_bool().unwrap_or(false),
                        Err(e) => {
                            log!("[ERROR] Failed to store bloodhound instance: {:?}", e);
                            false
                        }
                    };
                    
                    if !bloodhound_stored {
                        log!("[ERROR] Failed to store bloodhound instance in registry");
                        return;
                    }

                    // Check again if component is still alive
                    if !closures_clone.borrow().is_alive.load(Ordering::SeqCst) || cancel_token.get() {
                        log!("[INIT] Component no longer alive before typeahead init, aborting: {}", component_id_clone);
                        return;
                    }
                    
                    // Initialize typeahead with safe closure handling
                    if let Err(e) = initialize_typeahead(&input, bloodhound, on_select_clone.clone(), node_ref_clone.clone(), &component_id_clone, closures_clone.clone()) {
                        log!("[ERROR] Failed to initialize typeahead: {:?}", e);
                        return;
                    }
                    
                    // Mark as initialized in the registry
                    let mark_initialized_script = format!(
                        r#"
                        if (window.typeaheadRegistry && window.typeaheadRegistry['{component_id}']) {{
                            window.typeaheadRegistry['{component_id}'].initialized = true;
                            console.log('[JS] Marked component {component_id} as initialized');
                            true
                        }} else {{
                            console.error('[JS] Component registry not found for {component_id}');
                            false
                        }}
                        "#,
                        component_id = component_id_clone
                    );
                    
                    match js_sys::eval(&mark_initialized_script) {
                        Ok(_) => log!("[INIT] Marked component as initialized in registry"),
                        Err(e) => log!("[ERROR] Failed to mark as initialized: {:?}", e),
                    }
                    
                    // Only set initialized if component is still alive
                    if closures_clone.borrow().is_alive.load(Ordering::SeqCst) && !cancel_token.get() {
                        // Use a try_update to safely update the signal
                        if let Some(owner) = Owner::current() {
                            let _ = try_with_owner(owner, move || {
                                set_initialized.set(true);
                            });
                        } else {
                            log!("[INIT] No Leptos owner when setting initialized, aborting");
                        }
                    }
                    break;
                }
                
                // Check if component is still alive before sleeping
                if !closures_clone.borrow().is_alive.load(Ordering::SeqCst) || cancel_token.get() {
                    log!("[INIT] Component no longer alive during retry loop, aborting: {}", component_id_clone);
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
            
            // Perform JavaScript cleanup to prevent any further calls to Rust closures
            let js_cleanup_script = format!(
                r#"
                // Perform cleanup in JavaScript first
                if (window.typeaheadRegistry && window.typeaheadRegistry['{component_id}']) {{
                    console.log('[JS] Starting cleanup for component {component_id}');
                    
                    // Clean up the component
                    const result = window.typeaheadRegistry['{component_id}'].cleanup();
                    
                    // DEFENSIVE: Explicitly null out global handlers even if cleanup fails
                    if (window.rustSelectHandler_{component_id_safe}) {{
                        window.rustSelectHandler_{component_id_safe} = null;
                        console.log('[JS] Nulled rustSelectHandler_{component_id_safe} during cleanup');
                    }}
                    
                    if (window.rustFetchHandler_{component_id_safe}) {{
                        window.rustFetchHandler_{component_id_safe} = null;
                        console.log('[JS] Nulled rustFetchHandler_{component_id_safe} during cleanup');
                    }}
                    
                    // Remove from registry
                    delete window.typeaheadRegistry['{component_id}'];
                    
                    console.log('[JS] Component {component_id} removed from registry');
                    result
                }} else {{
                    console.warn('[JS] Component {component_id} not found in registry during cleanup');
                    false
                }}
                "#,
                component_id = component_id_for_cleanup,
                component_id_safe = component_id_for_cleanup.replace("-", "_")
            );
            
            // Execute JavaScript cleanup
            match js_sys::eval(&js_cleanup_script) {
                Ok(result) => {
                    if let Some(success) = result.as_bool() {
                        log!("[CLEANUP] JavaScript cleanup {}", if success { "successful" } else { "failed" });
                    }
                },
                Err(e) => log!("[CLEANUP] JavaScript cleanup error: {:?}", e),
            }
            
            // Now clean up Rust resources
            if let Ok(mut closures) = closures_for_cleanup.try_borrow_mut() {
                closures.cleanup();
            } else {
                log!("[CLEANUP] Warning: Could not borrow closures for cleanup");
            }
            
            log!("[CLEANUP] TypeaheadInput component cleanup completed");
        });
    }

    // CSS styles for the typeahead input
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
                if let Some(callback) = &on_input {
                    callback.call(value.clone());
                }
            }
        />
    }
}

// Only include these functions when targeting wasm
#[cfg(target_arch = "wasm32")]
fn initialize_bloodhound(
    fetch: Callback<String, Vec<WikidataSuggestion>>, 
    component_id: &str,
    closures: Rc<RefCell<TypeaheadClosures>>
) -> Result<JsValue, JsValue> {
    log!("[BLOODHOUND] Initializing for component: {}", component_id);
    
    let bloodhound_options = Object::new();
    
    // Get a clone of the is_alive flag for use in the closure
    let is_alive = closures.borrow().get_alive_flag();
    
    // Register the fetch handler in the component registry
    let register_fetch_handler_script = format!(
        r#"
        if (window.typeaheadRegistry && window.typeaheadRegistry['{component_id}']) {{
            window.typeaheadRegistry['{component_id}'].registerHandler('fetch', function(query, syncFn, asyncFn) {{
                // This function will be called by the transport function
                if (window.rustFetchHandler_{component_id_safe}) {{
                    try {{
                        window.rustFetchHandler_{component_id_safe}(query, syncFn, asyncFn);
                    }} catch (e) {{
                        console.error('[JS] Error calling Rust fetch handler:', e);
                        syncFn([]);
                    }}
                }} else {{
                    console.error('[JS] Rust fetch handler not found');
                    syncFn([]);
                }}
            }});
            console.log('[JS] Registered fetch handler for {component_id}');
            true
        }} else {{
            console.error('[JS] Component registry not found for {component_id}');
            false
        }}
        "#,
        component_id = component_id,
        component_id_safe = component_id.replace("-", "_")
    );
    
    let handler_registered = match js_sys::eval(&register_fetch_handler_script) {
        Ok(result) => result.as_bool().unwrap_or(false),
        Err(e) => {
            log!("[ERROR] Failed to register fetch handler: {:?}", e);
            return Err(JsValue::from_str("Failed to register fetch handler"));
        }
    };
    
    if !handler_registered {
        return Err(JsValue::from_str("Failed to register fetch handler in registry"));
    }
    
    // Create a closure that will be called by Bloodhound to fetch suggestions
    let remote_fn = Closure::wrap(Box::new({
        // Clone these values for use in the closure
        let is_alive = is_alive.clone();
        let fetch = fetch.clone();
        
        move |query: JsValue, sync: Function, _async_fn: Function| {
            // First check if the component is still alive
            if !is_alive.load(Ordering::SeqCst) {
                log!("[BLOODHOUND] Component no longer alive, aborting fetch");
                // Call sync with empty results to avoid JS errors
                let empty_results = Array::new();
                let _ = sync.call1(&JsValue::NULL, &empty_results);
                return;
            }
            
            let query_str = query.as_string().unwrap_or_default();
            log!("[BLOODHOUND] Fetching suggestions for: {}", query_str);
            
            // Defensive: check if we have a valid Leptos owner
            let owner = match Owner::current() {
                Some(owner) => owner,
                None => {
                    log!("[BLOODHOUND] No Leptos owner, aborting fetch");
                    let empty_results = Array::new();
                    let _ = sync.call1(&JsValue::NULL, &empty_results);
                    return;
                }
            };
            
            // Safely call the fetch callback
            let suggestions = match try_with_owner(owner, {
                // Clone these values for the inner closure
                let is_alive = is_alive.clone();
                let fetch = fetch.clone();
                let query_str = query_str.clone();
                
                move || {
                    // Check again if component is still alive
                    if !is_alive.load(Ordering::SeqCst) {
                        log!("[BLOODHOUND] Component no longer alive before fetch, aborting");
                        return Vec::new();
                    }
                    
                    fetch.call(query_str.clone())
                }
            }) {
                Ok(suggs) => suggs,
                Err(e) => {
                    log!("[ERROR] Failed to fetch suggestions: {:?}", e);
                    Vec::new()
                }
            };
            
            // Check again if component is still alive
            if !is_alive.load(Ordering::SeqCst) {
                log!("[BLOODHOUND] Component no longer alive after fetch, aborting processing");
                let empty_results = Array::new();
                let _ = sync.call1(&JsValue::NULL, &empty_results);
                return;
            }
            
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
            
            // Final check if component is still alive
            if !is_alive.load(Ordering::SeqCst) {
                log!("[BLOODHOUND] Component no longer alive before returning results, aborting");
                let empty_results = Array::new();
                let _ = sync.call1(&JsValue::NULL, &empty_results);
                return;
            }
            
            log!("[BLOODHOUND] Processed suggestions: {} items", js_suggestions.length());
            
            // Call the sync function with the suggestions
            let _ = sync.call1(&JsValue::NULL, &js_suggestions);
        }
    }) as Box<dyn FnMut(JsValue, Function, Function)>);

    // Store the Rust fetch handler globally
    let rust_handler_name = format!("rustFetchHandler_{}", component_id.replace("-", "_"));
    js_sys::Reflect::set(
        &js_sys::global(),
        &rust_handler_name.into(),
        remote_fn.as_ref().unchecked_ref()
    ).map_err(|e| {
        log!("[ERROR] Failed to store Rust fetch handler: {:?}", e);
        e
    })?;
    
    // Configure the remote options
    let remote_config = Object::new();
    
    // Set transport function to use our registry-based handler
    let transport_fn = js_sys::Function::new_with_args(
        "query, syncResults, asyncResults",
        &format!(r#"
        // DEFENSIVE: Check if registry exists and component is alive
        if (!window.typeaheadRegistry || !window.typeaheadRegistry['{component_id}']) {{
            console.warn('[JS] Component registry not found for {component_id}, returning empty results');
            syncResults([]);
            return;
        }}
        
        // DEFENSIVE: Check alive flag explicitly
        if (!window.typeaheadRegistry['{component_id}'].alive) {{
            console.warn('[JS] Component {component_id} is no longer alive, returning empty results');
            syncResults([]);
            return;
        }}
        
        // Call our handler through the registry
        try {{
            window.typeaheadRegistry['{component_id}'].callHandler('fetch', query, syncResults, asyncResults);
        }} catch (e) {{
            console.error('[JS] Error calling fetch handler through registry:', e);
            syncResults([]);
        }}
        "#, component_id = component_id)
    );
    
    Reflect::set(
        &remote_config,
        &"transport".into(),
        &transport_fn
    ).map_err(|e| {
        log!("[ERROR] Failed to set transport function: {:?}", e);
        e
    })?;
    
    // Set a dummy URL (not actually used with custom transport)
    Reflect::set(
        &remote_config,
        &"url".into(),
        &JsValue::from_str("/dummy?query=%QUERY")
    ).map_err(|e| {
        log!("[ERROR] Failed to set URL: {:?}", e);
        e
    })?;
    
    // Set rate limiting to prevent too many requests
    Reflect::set(
        &remote_config,
        &"rateLimitWait".into(),
        &JsValue::from(300)
    ).map_err(|e| {
        log!("[ERROR] Failed to set rate limit: {:?}", e);
        e
    })?;
    
    // Set the wildcard for query replacement
    Reflect::set(
        &remote_config,
        &"wildcard".into(),
        &JsValue::from_str("%QUERY")
    ).map_err(|e| {
        log!("[ERROR] Failed to set wildcard: {:?}", e);
        e
    })?;
    
    // Add the remote config to the options
    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).map_err(|e| {
        log!("[ERROR] Failed to set remote config: {:?}", e);
        e
    })?;
    
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
    ).map_err(|e| {
        log!("[ERROR] Failed to set datumTokenizer: {:?}", e);
        e
    })?;
    
    Reflect::set(
        &bloodhound_options, 
        &"queryTokenizer".into(), 
        &tokenizer
    ).map_err(|e| {
        log!("[ERROR] Failed to set queryTokenizer: {:?}", e);
        e
    })?;
    
    // Create and initialize the Bloodhound instance
    let bloodhound = Bloodhound::new(&bloodhound_options.into());
    bloodhound.initialize(true);
    
    // Store the closure in our struct instead of forgetting it
    if let Ok(mut closures_mut) = closures.try_borrow_mut() {
        closures_mut.remote_fn_closure = Some(remote_fn);
    } else {
        log!("[ERROR] Failed to store remote_fn_closure: could not borrow closures");
        return Err(JsValue::from_str("Failed to store remote_fn_closure"));
    }
    
    // Return the Bloodhound instance
    Ok(bloodhound.into())
}

// Only include this function when targeting wasm
#[cfg(target_arch = "wasm32")]
fn initialize_typeahead(
    input: &HtmlInputElement,
    bloodhound: JsValue,
    on_select: Callback<WikidataSuggestion>,
    node_ref: NodeRef<Input>,
    component_id: &str,
    closures: Rc<RefCell<TypeaheadClosures>>,
) -> Result<(), JsValue> {
    log!("[TYPEAHEAD] Initializing for input: {} ({})", input.id(), component_id);
    let input_id = format!("typeahead-input-{}", component_id);
    input.set_id(&input_id);

    // Get a clone of the is_alive flag for use in the closure
    let is_alive = closures.borrow().get_alive_flag();

    // Register the selection handler in the component registry
    let register_selection_handler_script = format!(
        r#"
        if (window.typeaheadRegistry && window.typeaheadRegistry['{component_id}']) {{
            window.typeaheadRegistry['{component_id}'].registerHandler('select', function(event, suggestion) {{
                // This function will be called by the typeahead:select event
                if (window.rustSelectHandler_{component_id_safe}) {{
                    try {{
                        window.rustSelectHandler_{component_id_safe}(event, suggestion);
                    }} catch (e) {{
                        console.error('[JS] Error calling Rust select handler:', e);
                    }}
                }} else {{
                    console.error('[JS] Rust select handler not found');
                }}
            }});
            console.log('[JS] Registered selection handler for {component_id}');
            true
        }} else {{
            console.error('[JS] Component registry not found for {component_id}');
            false
        }}
        "#,
        component_id = component_id,
        component_id_safe = component_id.replace("-", "_")
    );
    
    let handler_registered = match js_sys::eval(&register_selection_handler_script) {
        Ok(result) => result.as_bool().unwrap_or(false),
        Err(e) => {
            log!("[ERROR] Failed to register selection handler: {:?}", e);
            return Err(JsValue::from_str("Failed to register selection handler"));
        }
    };
    
    if !handler_registered {
        return Err(JsValue::from_str("Failed to register selection handler in registry"));
    }
    
    // Create selection handler closure
    let closure = Closure::wrap(Box::new({
        // Clone these values for use in the closure
        let is_alive = is_alive.clone();
        let on_select = on_select.clone();
        let node_ref = node_ref.clone();
        
        move |_event: web_sys::Event, suggestion: JsValue| {
            // First check if the component is still alive
            if !is_alive.load(Ordering::SeqCst) {
                log!("[TYPEAHEAD] Component no longer alive, aborting selection handler");
                return;
            }
            // Defensive: check if we have a valid Leptos owner
            let owner = match Owner::current() {
                Some(owner) => owner,
                None => {
                    log!("[TYPEAHEAD] No Leptos owner, aborting selection handler");
                    return;
                }
            };

            log!("[TYPEAHEAD] Selection made");
            
            // Safely call the on_select callback
            let _ = try_with_owner(owner, {
                // Clone these values again for the inner closure
                let is_alive = is_alive.clone();
                let on_select = on_select.clone();
                let node_ref = node_ref.clone();
                let suggestion = suggestion.clone();
                
                move || {
                    // Check again if component is still alive
                    if !is_alive.load(Ordering::SeqCst) {
                        log!("[TYPEAHEAD] Component no longer alive during selection callback, aborting");
                        return;
                    }
                    
                    // Try to get the full suggestion from the suggestion object
                    if let Some(full_suggestion) = js_sys::Reflect::get(&suggestion, &"fullSuggestion".into()).ok() {
                        if let Ok(data) = full_suggestion.into_serde::<WikidataSuggestion>() {
                            log!("[TYPEAHEAD] Selected suggestion: {:?}", data);
                            
                            // Final check before calling callback
                            if !is_alive.load(Ordering::SeqCst) {
                                log!("[TYPEAHEAD] Component no longer alive before callback, aborting");
                                return;
                            }
                            
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
                        
                        // Final check before calling callback
                        if !is_alive.load(Ordering::SeqCst) {
                            log!("[TYPEAHEAD] Component no longer alive before fallback callback, aborting");
                            return;
                        }
                        
                        on_select.call(data.clone());
                        
                        if let Some(input) = node_ref.get() {
                            input.set_value(&data.label);
                        }
                    } else {
                        log!("[ERROR] Failed to deserialize suggestion");
                    }
                }
            });
        }
    }) as Box<dyn FnMut(web_sys::Event, JsValue)>);

    // Store the Rust selection handler globally with a component-specific name
    let rust_handler_name = format!("rustSelectHandler_{}", component_id.replace("-", "_"));
    js_sys::Reflect::set(
        &js_sys::global(),
        &rust_handler_name.into(),
        closure.as_ref().unchecked_ref(),
    ).map_err(|e| {
        log!("[ERROR] Failed to store Rust selection handler: {:?}", e);
        e
    })?;
    
    // Store the closure in our struct instead of forgetting it
    if let Ok(mut closures_mut) = closures.try_borrow_mut() {
        closures_mut.selection_closure = Some(closure);
    } else {
        log!("[ERROR] Failed to store selection_closure: could not borrow closures");
        return Err(JsValue::from_str("Failed to store selection_closure"));
    }

    // Initialization script with enhanced logging and error handling
    let init_script = format!(
        r#"
        console.log('[JS] Starting Typeahead init for #{input_id}');
        try {{
            // DEFENSIVE: Check if registry exists and component is alive
            if (!window.typeaheadRegistry || !window.typeaheadRegistry['{component_id}']) {{
                throw new Error('Component not found in registry: {component_id}');
            }}
            
            // DEFENSIVE: Check alive flag explicitly
            if (!window.typeaheadRegistry['{component_id}'].alive) {{
                throw new Error('Component {component_id} is no longer alive');
            }}
            
            // Get the bloodhound instance from the registry
            var bloodhound = window.typeaheadRegistry['{component_id}'].bloodhound;
            if (!bloodhound) {{
                throw new Error('Bloodhound instance not found in registry');
            }}
            
            // Initialize typeahead with error handling
            var $input = $('#{input_id}');
            if ($input.length === 0) {{
                throw new Error('Input element not found: #{input_id}');
            }}
            
            // DEFENSIVE: Remove any existing typeahead to prevent duplicate handlers
            $input.typeahead('destroy');
            
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
                    source: function(query, syncResults, asyncResults) {{
                        // DEFENSIVE: Check if registry exists and component is alive
                        if (!window.typeaheadRegistry || !window.typeaheadRegistry['{component_id}']) {{
                            console.warn('[JS] Component registry not found for {component_id}, returning empty results');
                            syncResults([]);
                            return;
                        }}
                        
                        // DEFENSIVE: Check alive flag explicitly
                        if (!window.typeaheadRegistry['{component_id}'].alive) {{
                            console.warn('[JS] Component {component_id} is no longer alive, returning empty results');
                            syncResults([]);
                            return;
                        }}
                        
                        // Call the fetch handler through the registry
                        window.typeaheadRegistry['{component_id}'].callHandler('fetch', query, syncResults, asyncResults);
                    }},
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
                // DEFENSIVE: Check if registry exists and component is alive
                if (!window.typeaheadRegistry || !window.typeaheadRegistry['{component_id}']) {{
                    console.warn('[JS] Component registry not found for {component_id}, ignoring selection event');
                    return;
                }}
                
                // DEFENSIVE: Check alive flag explicitly
                if (!window.typeaheadRegistry['{component_id}'].alive) {{
                    console.warn('[JS] Component {component_id} is no longer alive, ignoring selection event');
                    return;
                }}
                
                if (!suggestion) {{
                    console.error('[JS] Selection event received with null suggestion');
                    return;
                }}
                
                console.log('[JS] Selection event received with suggestion:', suggestion);
                
                // Call the selection handler through the registry
                window.typeaheadRegistry['{component_id}'].callHandler('select', ev, suggestion);
            }});
            
            console.log('[JS] Typeahead initialized successfully for #{input_id}');
            true
        }} catch (e) {{
            console.error('[JS] Typeahead init error for #{input_id}:', e);
            false
        }}
        "#,
        component_id = component_id,
        input_id = input_id
    );

    log!("[RUST] Running initialization script for: {}", input_id);
    match js_sys::eval(&init_script) {
        Ok(result) => {
            if let Some(success) = result.as_bool() {
                if success {
                    log!("[RUST] Initialization script executed successfully");
                    Ok(())
                } else {
                    log!("[RUST] Initialization script failed");
                    Err(JsValue::from_str("Initialization script failed"))
                }
            } else {
                log!("[RUST] Initialization script returned non-boolean result");
                Ok(())
            }
        },
        Err(e) => {
            log!("[RUST] Eval error: {:?}", e);
            Err(e)
        }
    }
}
