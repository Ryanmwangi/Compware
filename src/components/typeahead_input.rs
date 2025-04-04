use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::item::WikidataSuggestion;
use js_sys::{Object, Array, Function, JSON, Reflect};
use leptos::html::Input; 
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement; 
use leptos::logging::log;
#[component]
pub fn TypeaheadInput(
    value: String,
    on_select: Callback<WikidataSuggestion>,
    fetch_suggestions: Callback<String, Vec<WikidataSuggestion>>,
    node_ref: NodeRef<Input>,
) -> impl IntoView {
    let (is_initialized, set_initialized) = create_signal(false);
    
    create_effect(move |_| {
        if let (Some(input), false) = (node_ref.get(), is_initialized.get()) {
            let bloodhound = initialize_bloodhound(fetch_suggestions.clone());
            initialize_typeahead(&input, bloodhound, on_select.clone(), node_ref.clone());
            set_initialized.set(true);
        }
    });

    view! {
        <input
            type="text"
            class="typeahead"
            prop:value=value
            node_ref=node_ref
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
    
    // Store the Closure in a variable to prevent premature garbage collection
    let remote_fn = Closure::wrap(Box::new(move |query: String, sync: js_sys::Function| {
        log!("Fetching suggestions for: {}", query);
        let suggestions = fetch.call(query.clone());
        let array = Array::new();
        for suggestion in &suggestions {
            let obj = Object::new();
            Reflect::set(&obj, &"label".into(), &suggestion.label.clone().into()).unwrap();
            Reflect::set(&obj, &"value".into(), &suggestion.id.clone().into()).unwrap();
            array.push(&obj);
        }
        sync.call1(&JsValue::NULL, &array).unwrap();
    }) as Box<dyn Fn(String, js_sys::Function)>);

    // Configure Bloodhound 
    let remote_config = Object::new();
    Reflect::set(&remote_config, &"url".into(), &"".into()).unwrap();
    Reflect::set(&remote_config, &"wildcard".into(), &"%QUERY".into()).unwrap();
    Reflect::set(&remote_config, &"prepare".into(), &remote_fn.as_ref()).unwrap();
    Reflect::set(&remote_config, &"rateLimitWait".into(), &JsValue::from(300)).unwrap();

    Reflect::set(&bloodhound_options, &"remote".into(), &remote_config).unwrap();
    Reflect::set(&bloodhound_options, &"queryTokenizer".into(), &JsValue::from("whitespace")).unwrap();
    Reflect::set(&bloodhound_options, &"datumTokenizer".into(), &JsValue::from("whitespace")).unwrap();

    let bloodhound = Bloodhound::new(&bloodhound_options.into());
    bloodhound.initialize(true);
    
    // Prevent Closure from being dropped
    remote_fn.forget();
    
    bloodhound.into()
}

fn initialize_typeahead(
    input: &HtmlInputElement,
    bloodhound: JsValue,
    on_select: Callback<WikidataSuggestion>,
    node_ref: NodeRef<Input>,
) {
    // input event handler for direct typing
    let node_ref_clone = node_ref.clone();
    let input_handler = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if let Some(input) = node_ref_clone.get() {
            let value = input.value();
            log!("Input updated: {}", value);
            // Create synthetic change event for Leptos
            let event = web_sys::CustomEvent::new("input").unwrap();
            input.dispatch_event(&event).unwrap();
        }
    }) as Box<dyn FnMut(_)>);

    input.add_event_listener_with_callback(
        "input",
        input_handler.as_ref().unchecked_ref()
    ).unwrap();
    input_handler.forget();

    let typeahead_options = Object::new();
    Reflect::set(&typeahead_options, &"hint".into(), &JsValue::TRUE).unwrap();
    Reflect::set(&typeahead_options, &"highlight".into(), &JsValue::TRUE).unwrap();
    Reflect::set(&typeahead_options, &"minLength".into(), &JsValue::from(1)).unwrap();

    // Bloodhound remote configuration
    let bloodhound_ref = bloodhound.unchecked_ref::<Bloodhound>();
    let remote_config = Object::new();
    Reflect::set(&remote_config, &"prepare".into(), &JsValue::from_str("function(q) { return { q: q }; }")).unwrap();
    Reflect::set(&remote_config, &"transform".into(), &js_sys::Function::new_with_args("response", "return response;")).unwrap();

    // Update dataset configuration
    let dataset = Object::new();
    Reflect::set(&dataset, &"source".into(), &bloodhound_ref.tt_adapter()).unwrap();
    Reflect::set(&dataset, &"display".into(), &"label".into()).unwrap();
    Reflect::set(&dataset, &"limit".into(), &JsValue::from(10)).unwrap();

    // Create proper templates
    let templates = Object::new();
    Reflect::set(&templates, &"suggestion".into(), &js_sys::eval(r#"
        function(data) {
            return '<div class="suggestion-item">' +
                   '<strong>' + data.label + '</strong>' +
                   (data.description ? '<br><small>' + data.description + '</small>' : '') +
                   '</div>';
        }
    "#).unwrap()).unwrap();

    Reflect::set(&dataset, &"templates".into(), &templates).unwrap();

    // Typeahead initialization using jQuery
    let init_script = format!(
        r#"(function() {{
            $('#{}').typeahead({}, {});
        }})"#,
        input.id(),
        JSON::stringify(&typeahead_options).unwrap(),
        JSON::stringify(&dataset).unwrap()
    );
    
    let _ = js_sys::eval(&init_script).unwrap();

    // Handle selection events
    let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Some(selected) = event.target() {
            let js_value = selected.unchecked_into::<JsValue>();
            let data: WikidataSuggestion = js_sys::JSON::parse(
                &js_sys::JSON::stringify(&js_value)
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
            ).unwrap()
            .into_serde()
            .unwrap();
            let data_clone = data.clone();
            
            on_select.call(data);
            // Explicitly update the input value
            if let Some(input) = node_ref.get() {
                input.set_value(&data_clone.label);
            }
        }
    }) as Box<dyn FnMut(_)>);

    input.add_event_listener_with_callback(
        "typeahead:select",
        closure.as_ref().unchecked_ref()
    ).unwrap();
    closure.forget();
}