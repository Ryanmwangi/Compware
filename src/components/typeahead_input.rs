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
        <input
            type="text"
            class="typeahead"
            prop:value=value
            node_ref=node_ref
            on:focus=move |_| log!("[FOCUS] Name input focused")
            on:blur=move |_| log!("[FOCUS] Name input blurred")
            on:input=move |ev| {
                let value = event_target_value(&ev);
                log!("[INPUT] Value changed: {}", value);
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
    
    let remote_fn = Closure::wrap(Box::new(move |query: JsValue, sync: Function| {
        let query_str = query.as_string().unwrap_or_default();
        log!("[BLOODHOUND] Fetching suggestions for: {}", query_str);
        let suggestions = fetch.call(query_str.clone());
        log!("[BLOODHOUND] Received {} suggestions", suggestions.len());

        let array = Array::new();
        for suggestion in &suggestions {
            let obj = Object::new();
            Reflect::set(&obj, &"label".into(), &suggestion.label.clone().into()).unwrap_or_default();
            Reflect::set(&obj, &"value".into(), &suggestion.id.clone().into()).unwrap_or_default();
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
    
    // Wildcard function
    Reflect::set(
        &remote_config,
        &"wildcard".into(),
        &JsValue::from_str("%QUERY")
    ).unwrap();

    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).unwrap();

    // Tokenizer functions from Bloodhound
    let tokenizer = js_sys::eval(r#"Bloodhound.tokenizers.whitespace"#)
        .expect("Should get whitespace tokenizer");

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

    let dataset = Object::new();
    let bloodhound_ref = bloodhound.unchecked_ref::<Bloodhound>();
    
    Reflect::set(&dataset, &"source".into(), &bloodhound_ref.tt_adapter()).unwrap();
    Reflect::set(&dataset, &"display".into(), &"label".into()).unwrap();
    Reflect::set(&dataset, &"limit".into(), &JsValue::from(10)).unwrap();

    let templates = Object::new();
    let suggestion_fn = js_sys::Function::new_no_args(
        "return '<div class=\"suggestion-item\">' + data.label + '</div>';"
    );
    Reflect::set(&templates, &"suggestion".into(), &suggestion_fn.into()).unwrap();
    Reflect::set(&dataset, &"templates".into(), &templates).unwrap();

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
    
    let handler_name = format!("handler_{}", input_id);
    js_sys::Reflect::set(
        &js_sys::global(),
        &handler_name.clone().into(),
        closure.as_ref(),
    ).unwrap();
    closure.forget();

    // Corrected initialization script using bracket notation for handler
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
                    source: bloodhound.ttAdapter(),
                    display: 'label',
                    templates: {{
                        suggestion: function(data) {{
                            console.log('[JS] Rendering suggestion', data);
                            return $('<div>').text(data.label);
                        }}
                    }}
                }}
            ).on('typeahead:select', function(ev, suggestion) {{
                console.log('[JS] Selection event received');
                window['{handler}'](ev, suggestion);
            }});
            console.log('[JS] Typeahead initialized successfully');
        }} catch (e) {{
            console.error('[JS] Typeahead init error:', e);
        }}
        "#,
        id = input_id,
        handler = handler_name.replace('-', "_") // Replace hyphens to avoid JS issues
    );

    log!("[RUST] Initialization script: {}", init_script);
    if let Err(e) = js_sys::eval(&init_script) {
        log!("[RUST] Eval error: {:?}", e);
    }
}