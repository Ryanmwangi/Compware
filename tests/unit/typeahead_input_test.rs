use wasm_bindgen_test::*;
use leptos::*;
use wasm_bindgen::JsValue;
use std::time::Duration;
use gloo_timers::future::sleep;
use compareware::components::typeahead_input::TypeaheadInput;
use compareware::models::item::WikidataSuggestion;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_typeahead_initialization() {
    // Setup
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("test-container");
    
    // Track initialization
    let init_called = create_rw_signal(false);
    
    // Create a test component
    let test_component = move || {
        let node_ref = create_node_ref::<html::Input>();
        
        // Mock callbacks
        let on_select = Callback::new(move |suggestion: WikidataSuggestion| {
            log!("Selected: {}", suggestion.label);
        });
        
        let fetch_suggestions = Callback::new(move |query: String| {
            log!("Fetching: {}", query);
            init_called.set(true);
            vec![]
        });
        
        view! {
            <div>
                <TypeaheadInput
                    value="".to_string()
                    on_select=on_select
                    fetch_suggestions=fetch_suggestions
                    node_ref=node_ref
                />
            </div>
        }
    };
    
    // Mount the component
    mount_to(&container, test_component);
    
    // Wait for initialization
    for _ in 0..10 {
        if init_called.get() {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    // Verify initialization
    assert!(init_called.get(), "Initialization callback was not called");
    
    // Cleanup
    document.body().unwrap().remove_child(&container).unwrap();
}

#[wasm_bindgen_test]
async fn test_typeahead_cleanup() {
    // Setup
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("cleanup-test-container");
    
    // Create a unique component ID for tracking
    let component_id = format!("test-typeahead-{}", uuid::Uuid::new_v4());
    
    // Create a test component
    let test_component = move || {
        let node_ref = create_node_ref::<html::Input>();
        
        // Mock callbacks
        let on_select = Callback::new(move |_: WikidataSuggestion| {});
        let fetch_suggestions = Callback::new(move |_: String| vec![]);
        
        view! {
            <div>
                <TypeaheadInput
                    value="".to_string()
                    on_select=on_select
                    fetch_suggestions=fetch_suggestions
                    node_ref=node_ref
                />
            </div>
        }
    };
    
    // Mount the component
    let dispose = mount_to(&container, test_component);
    
    // Wait for initialization
    sleep(Duration::from_millis(500)).await;
    
    // Check registry before unmount
    let registry_before = js_sys::eval(&format!(
        "window.typeaheadRegistry ? Object.keys(window.typeaheadRegistry).length : 0"
    )).unwrap();
    
    // Unmount the component
    dispose();
    
    // Wait for cleanup
    sleep(Duration::from_millis(500)).await;
    
    // Check if component was properly removed from registry
    let registry_after = js_sys::eval(&format!(
        "window.typeaheadRegistry ? Object.keys(window.typeaheadRegistry).length : 0"
    )).unwrap();
    
    // Registry should have one fewer entry
    assert!(
        registry_before.as_f64().unwrap() > registry_after.as_f64().unwrap(),
        "Component was not properly removed from registry"
    );
    
    // Cleanup
    document.body().unwrap().remove_child(&container).unwrap();
}

#[wasm_bindgen_test]
async fn test_rapid_mount_unmount() {
    // Setup
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("rapid-test-container");
    
    // Perform rapid mount/unmount cycles to test for race conditions
    for i in 0..5 {
        log!("Mount/unmount cycle {}", i);
        
        // Create a test component
        let test_component = move || {
            let node_ref = create_node_ref::<html::Input>();
            let on_select = Callback::new(move |_: WikidataSuggestion| {});
            let fetch_suggestions = Callback::new(move |_: String| vec![]);
            
            view! {
                <div>
                    <TypeaheadInput
                        value="".to_string()
                        on_select=on_select
                        fetch_suggestions=fetch_suggestions
                        node_ref=node_ref
                    />
                </div>
            }
        };
        
        // Mount
        let dispose = mount_to(&container, test_component);
        
        // Wait briefly
        sleep(Duration::from_millis(50)).await;
        
        // Unmount
        dispose();
        
        // Wait briefly
        sleep(Duration::from_millis(50)).await;
    }
    
    // Wait for any pending cleanup
    sleep(Duration::from_millis(500)).await;
    
    // Check if registry is clean
    let registry_size = js_sys::eval(
        "window.typeaheadRegistry ? Object.keys(window.typeaheadRegistry).length : 0"
    ).unwrap();
    
    // Registry should be empty or at least not growing
    assert!(
        registry_size.as_f64().unwrap() < 5.0,
        "Registry has too many entries after rapid mount/unmount cycles"
    );
    
    // Cleanup
    document.body().unwrap().remove_child(&container).unwrap();
}

// Helper function to mount a component to a container
fn mount_to(container: &web_sys::Element, component: impl FnOnce() -> View + 'static) -> impl FnOnce() {
    let runtime = create_runtime();
    let view = component();
    leptos::mount_to_with_runtime(container, || view, runtime.clone());
    
    move || {
        runtime.dispose();
    }
}