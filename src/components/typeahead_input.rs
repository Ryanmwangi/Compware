use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::item::WikidataSuggestion;
use js_sys::{Object, Array, Function, JSON, Reflect};
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
                if let Some(input) = node_ref.get() {
                    // Correct DOM element access using JsCast
                    let dom_input: &web_sys::HtmlInputElement = &*input;
                    let id = dom_input.id();
                    let _ = js_sys::eval(&format!("console.log('JS Value:', $('#{}').val())", id));
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
    
    // Configure Bloodhound remote with proper parameters
    let remote_fn = Closure::wrap(Box::new(move |query: String, sync: Function| {
        log!("[BLOODHOUND] Fetching suggestions for: {}", query);
        let suggestions = fetch.call(query.clone());
        log!("[BLOODHOUND] Received {} suggestions", suggestions.len());

        let array = Array::new();
        for suggestion in &suggestions {
            let obj = Object::new();
            Reflect::set(&obj, &"label".into(), &suggestion.label.clone().into()).unwrap();
            Reflect::set(&obj, &"value".into(), &suggestion.id.clone().into()).unwrap();
            array.push(&obj);
        }

        sync.call1(&JsValue::NULL, &array).unwrap();
    }) as Box<dyn Fn(String, Function)>);

    let remote_config = Object::new();
    Reflect::set(
        &remote_config,
        &"prepare".into(),
        &js_sys::eval(&format!(
            "function(query, callback) {{ 
                return {}(query, callback); 
            }}",
            remote_fn.as_ref().unchecked_ref::<js_sys::Function>().to_string()
        )).unwrap()
    ).unwrap();
    
    Reflect::set(
        &remote_config,
        &"wildcard".into(),
        &JsValue::from_str("%QUERY")
    ).unwrap();

    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).unwrap();
    Reflect::set(&bloodhound_options, &"datumTokenizer".into(), &JsValue::from_str("whitespace")).unwrap();
    Reflect::set(&bloodhound_options, &"queryTokenizer".into(), &JsValue::from_str("whitespace")).unwrap();

    let bloodhound = Bloodhound::new(&bloodhound_options.into());
    bloodhound.initialize(true);
    remote_fn.forget();

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

    // Create and register the closure
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event, suggestion: JsValue| {
        log!("[TYPEAHEAD] Selection made");
        let data: WikidataSuggestion = suggestion.into_serde().unwrap();
        on_select.call(data.clone());
        
        if let Some(input) = node_ref.get() {
            input.set_value(&data.label);
        }
    }) as Box<dyn FnMut(web_sys::Event, JsValue)>);

    // Register the closure in the JS global scope
    let handler_name = format!("handler_{}", input_id);
    let handler_name_global = handler_name.clone();
    let global = js_sys::global();
    Reflect::set(
        &global, 
        &handler_name_global.into(),
        &closure.as_ref()
    ).unwrap();

    // Typeahead initialization using jQuery
    let init_script = format!(
        r#"
        (function() {{
            console.log('[TYPEAHEAD] Initializing for #{id}');
            $('#{id}').typeahead(
                {{
                    hint: true,
                    highlight: true,
                    minLength: 1
                }},
                {dataset}
            ).on('typeahead:select', function(ev, suggestion) {{
                console.log('[TYPEAHEAD] Select event triggered');
                {handler}(ev, suggestion);
            }});
            console.log('[TYPEAHEAD] Initialization complete for #{id}');
        }})();
        "#,
        id = input_id,
        dataset = JSON::stringify(&dataset).unwrap(),
        handler = handler_name
    );

    log!("[TYPEAHEAD] Init script: {}", init_script);
    let _ = js_sys::eval(&init_script).unwrap();
    closure.forget();
}