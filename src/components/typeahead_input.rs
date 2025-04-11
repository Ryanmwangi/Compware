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
                let _ = js_sys::eval("console.log('jQuery version:', $.fn.jquery)");
                let _ = js_sys::eval("console.log('Typeahead version:', $.fn.typeahead ? 'loaded' : 'missing')");
                // Add debug check for Typeahead instance
                if let Some(input) = node_ref.get() {
                    let dom_input: web_sys::HtmlInputElement = input.unchecked_into();
                    let id = dom_input.id();
                    let _ = js_sys::eval(&format!(
                        "console.log('Typeahead instance for #{id}:', $('#{id}').data('ttTypeahead'))",
                        id = id
                    ));
                }
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
    
    let remote_fn = Closure::wrap(Box::new(move |query: JsValue, sync: Function| {
        let query_str = query.as_string().unwrap_or_default();
        log!("[BLOODHOUND] Fetching suggestions for: {}", query_str);
        let suggestions = fetch.call(query_str.clone());
        
        let array = Array::new();
        for suggestion in &suggestions {
            let obj = Object::new();
            
            // Create nested display structure matching API response
            let display = Object::new();
            
            let label_obj = Object::new();
            Reflect::set(&label_obj, &"value".into(), &suggestion.label.clone().into()).unwrap();
            Reflect::set(&display, &"label".into(), &label_obj).unwrap();
            
            let desc_obj = Object::new();
            Reflect::set(&desc_obj, &"value".into(), &suggestion.description.clone().into()).unwrap();
            Reflect::set(&display, &"description".into(), &desc_obj).unwrap();
            
            Reflect::set(&obj, &"display".into(), &display).unwrap();
            
            // Add flat fields as fallback
            Reflect::set(&obj, &"label".into(), &suggestion.label.clone().into()).unwrap();
            Reflect::set(&obj, &"description".into(), &suggestion.description.clone().into()).unwrap();
            
            log!("[BLOODHOUND] Constructed suggestion object: {:?}", obj);
            array.push(&obj);
        }
        let _ = sync.call1(&JsValue::NULL, &array);
    }) as Box<dyn Fn(JsValue, Function)>);

    let remote_config = Object::new();

    // Url function
    Reflect::set(
        &remote_config,
        &"url".into(),
        &JsValue::from_str("/dummy?query=%QUERY")
    ).unwrap();
    
    // Prepare function
    Reflect::set(
        &remote_config,
        &"prepare".into(),
        remote_fn.as_ref().unchecked_ref()
    ).unwrap();

    // Rate limiting
    Reflect::set(
        &remote_config,
        &"rateLimitWait".into(),
        &JsValue::from(300)
    ).unwrap();

    // Response filter to prevent HTML parsing errors
    let filter_fn = js_sys::Function::new_no_args(
        "return function(response) { return response || []; }"
    );
    Reflect::set(
        &remote_config,
        &"filter".into(),
        &filter_fn
    ).unwrap();
    
    // Wildcard function
    Reflect::set(
        &remote_config,
        &"wildcard".into(),
        &JsValue::from_str("%QUERY")
    ).unwrap();

    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).unwrap();

    // Tokenizer functions from Bloodhound
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

    let bloodhound = Bloodhound::new(&bloodhound_options.into());
    bloodhound.initialize(true);
    remote_fn.forget();

    // Explicit retention
    js_sys::Reflect::set(
        &js_sys::global(),
        &"bloodhoundInstance".into(),
        &bloodhound
    ).unwrap();
    
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
        if let Ok(data) = suggestion.into_serde::<WikidataSuggestion>() {
            on_select.call(data.clone());
            if let Some(input) = node_ref.get() {
                input.set_value(&data.label);
            }
        } else {
            log!("[ERROR] Failed to deserialize suggestion");
        }
    }) as Box<dyn FnMut(web_sys::Event, JsValue)>);
    
    // Register global handler
    let handler_name = format!("handler_{}", input_id);
    js_sys::Reflect::set(
        &js_sys::global(),
        &handler_name.clone().into(),
        closure.as_ref(),
    ).unwrap();
    closure.forget();

    // Initialization script
    let init_script = format!(
        r#"
        console.log('[JS] Starting Typeahead init for #{id}');
        try {{
            var bloodhound = window.bloodhoundInstance;
            $('#{id}').typeahead(
                {{
                    hint: true,
                    highlight: true,
                    minLength: 1
                }},
                {{
                    name: 'suggestions',
                    display: 'label',
                    source: bloodhound.ttAdapter(),
                    templates: {{
                        suggestion: function(data) {{
                            // Handle nested Wikidata structure
                            var label = data.label || '';
                            var description = data.description || '';
                            
                            // If nested display exists, use those values
                            if (data.display) {{
                                label = data.display.label?.value || label;
                                description = data.display.description?.value || description;
                            }}
                            
                            return $('<div>')
                                .addClass('suggestion-item')
                                .append($('<div>').addClass('label').text(label))
                                .append($('<div>').addClass('description').text(description));
                        }},
                        empty: $('<div>').addClass('empty-suggestion').text('No matches found')
                    }}
                }}
            )
            .on('typeahead:asyncreceive', function(ev, dataset, suggestions) {{
                console.log('[JS] Received suggestions:', suggestions);
                if (suggestions && suggestions.length > 0) {{
                    $(this).data('ttTypeahead').dropdown.open();
                }}
            }})
            .on('typeahead:select', function(ev, suggestion) {{
                console.log('[JS] Selection data:', JSON.stringify(suggestion, null, 2));
                window['{handler}'](ev, suggestion);
            }});
        }} catch (e) {{
            console.error('[JS] Typeahead init error:', e);
        }}
        "#,
        id = input_id,
        handler = handler_name.replace('-', "_")
    );

    log!("[RUST] Initialization script: {}", init_script);
    if let Err(e) = js_sys::eval(&init_script) {
        log!("[RUST] Eval error: {:?}", e);
    }
}
