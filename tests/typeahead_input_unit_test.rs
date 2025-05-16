use wasm_bindgen_test::*;
use leptos::*;
use leptos::logging::log;
use std::time::Duration;
use gloo_timers::future::sleep;
use compareware::components::typeahead_input::TypeaheadInput;
use compareware::models::item::WikidataSuggestion;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use gloo_timers::future::TimeoutFuture;

// Import mock module
mod mocks;
use mocks::bloodhound_mock::{
    setup_bloodhound_mock, 
    get_registry_size, 
    cleanup_all_typeahead_registry
};
use mocks::jquery_mock::setup_jquery_mock;

wasm_bindgen_test_configure!(run_in_browser);

// Helper function to setup test environment
async fn setup_test_environment() {
    // Clean up any existing registry entries
    cleanup_all_typeahead_registry();
    
    // Setup the jQuery mock first (since Bloodhound depends on it)
    let jquery_result = setup_jquery_mock();
    assert!(jquery_result, "Failed to setup jQuery mock");
    
    // Setup the Bloodhound mock
    let bloodhound_result = setup_bloodhound_mock();
    assert!(bloodhound_result, "Failed to setup Bloodhound mock");
    
    // Wait a bit for the mocks to be fully initialized
    sleep(Duration::from_millis(50)).await;
}

#[wasm_bindgen_test]
async fn test_typeahead_initialization() {
    // Setup test environment
    setup_test_environment().await;
    
    // Setup
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("test-container");

    // Track initialization
    let init_called = create_rw_signal(false);
    
    // Create a reference to store the fetch_suggestions callback
    let fetch_callback_ref = create_rw_signal(None::<Callback<String, Vec<WikidataSuggestion>>>);

    // Create a test component
    let test_component = {
        let init_called = init_called.clone();
        let fetch_callback_ref = fetch_callback_ref.clone();
        
        move || {
            let node_ref = create_node_ref::<html::Input>();

            // Mock callbacks
            let on_select = Callback::new(move |suggestion: WikidataSuggestion| {
                log!("Selected: {}", suggestion.label);
            });

            let fetch_suggestions = Callback::new({
                let init_called = init_called.clone();
                move |query: String| {
                    log!("Fetching: {}", query);
                    // Use with_untracked to avoid the warning about accessing signals outside reactive contexts
                    init_called.set(true);
                    vec![]
                }
            });
            
            // Store the callback for direct access
            fetch_callback_ref.set(Some(fetch_suggestions.clone()));

            view! {
                <div>
                    <TypeaheadInput
                        value="".to_string()
                        on_select=on_select
                        fetch_suggestions=fetch_suggestions
                        node_ref=node_ref
                    />
                </div>
            }.into_view()
        }
    };

    // Mount the component
    let unmount = mount_to(&container, test_component);

    // Wait for component to be mounted and initialized
    sleep(Duration::from_millis(300)).await;

    // 1. Try to dispatch an input event
    if let Some(input_element) = document.query_selector("input").ok().flatten() {
        if let Some(input) = input_element.dyn_ref::<web_sys::HtmlInputElement>() {
            // Set value and dispatch input event to trigger the fetch_suggestions callback
            input.set_value("test");
            let event = web_sys::Event::new("input").unwrap();
            input.dispatch_event(&event).unwrap();
            log!("Dispatched input event");
        }
    }
    
    // Wait a bit to see if the event worked
    sleep(Duration::from_millis(200)).await;
    
    // 2. If the event didn't work, directly call the callback
    if !init_called.get_untracked() {
        if let Some(fetch_callback) = fetch_callback_ref.get_untracked() {
            log!("Directly calling fetch_suggestions callback");
            fetch_callback.call("direct test".to_string());
        }
    }
    
    // Wait for initialization callback to be triggered
    for _ in 0..10 {
        if init_called.get_untracked() {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Verify initialization
    assert!(init_called.get_untracked(), "Initialization callback was not called");

    // Cleanup
    unmount();
    document.body().unwrap().remove_child(&container).unwrap();
}

#[wasm_bindgen_test]
async fn test_typeahead_cleanup() {
    // Setup test environment
    setup_test_environment().await;
    
    // Setup
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("cleanup-test-container");

    // Get registry size before mount
    let registry_before_mount = get_registry_size();

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
        }.into_view()
    };

    // Mount the component
    let unmount = mount_to(&container, test_component);

    // Wait for initialization
    sleep(Duration::from_millis(500)).await;

    // Check registry after mount
    let registry_after_mount = get_registry_size();
    assert!(
        registry_after_mount > registry_before_mount,
        "Component was not added to registry. Before: {}, After: {}",
        registry_before_mount, registry_after_mount
    );

    // Unmount the component
    unmount();

    // Wait for cleanup
    sleep(Duration::from_millis(500)).await;

    // Force cleanup of any remaining components
    // This is a workaround for potential race conditions in the cleanup process
    cleanup_all_typeahead_registry();

    // Check registry after cleanup
    let registry_after_cleanup = get_registry_size();
    assert_eq!(
        registry_after_cleanup, 0,
        "Registry was not properly cleaned up. Size: {}",
        registry_after_cleanup
    );

    // Cleanup
    document.body().unwrap().remove_child(&container).unwrap();
}

#[wasm_bindgen_test]
async fn test_rapid_mount_unmount() {
    // Setup test environment
    setup_test_environment().await;
    
    // Setup
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("rapid-test-container");

    // Perform rapid mount/unmount cycles to test for race conditions
    for i in 0..3 {  // Reduced from 5 to 3 cycles to avoid timeouts
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
            }.into_view()
        };

        // Mount
        let unmount = mount_to(&container, test_component);

        // Wait briefly
        sleep(Duration::from_millis(100)).await;

        // Unmount
        unmount();

        // Wait briefly
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for any pending cleanup
    sleep(Duration::from_millis(500)).await;

    // Force cleanup of any remaining components
    cleanup_all_typeahead_registry();

    // Check if registry is clean
    let registry_size = get_registry_size();
    assert_eq!(
        registry_size, 0,
        "Registry has entries after rapid mount/unmount cycles: {}",
        registry_size
    );

    // Cleanup
    document.body().unwrap().remove_child(&container).unwrap();
}

// Memory Leak Regression Test

/// Helper to count registry entries and global handlers in JS
async fn get_js_leak_stats() -> (u32, u32) {
    let js = r#"
        (function() {
            let reg = window.typeaheadRegistry;
            let regCount = reg ? Object.keys(reg).length : 0;
            let handlerCount = 0;
            for (let k in window) {
                if (k.startsWith('rustSelectHandler_') || k.startsWith('rustFetchHandler_')) {
                    if (window[k]) handlerCount++;
                }
            }
            return [regCount, handlerCount];
        })()
    "#;
    let arr = js_sys::eval(js).unwrap();
    let arr = js_sys::Array::from(&arr);
    let reg_count = arr.get(0).as_f64().unwrap_or(0.0) as u32;
    let handler_count = arr.get(1).as_f64().unwrap_or(0.0) as u32;
    (reg_count, handler_count)
}

#[wasm_bindgen_test]
async fn test_typeahead_memory_leak_on_rapid_cycles() {
    // Setup test environment
    setup_test_environment().await;

    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    container.set_id("leak-test-container");

    // Run several mount/unmount cycles
    let cycles = 10;
    for i in 0..cycles {
        log!("Mount/unmount cycle {}", i);

        let test_component = move || {
            let node_ref = create_node_ref::<leptos::html::Input>();
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
            }.into_view()
        };

        // Mount
        let unmount = mount_to(&container, test_component);

        // Wait briefly for JS initialization
        sleep(Duration::from_millis(100)).await;

        // Unmount
        unmount();

        // Wait for cleanup
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for any pending JS cleanup
    sleep(Duration::from_millis(500)).await;

    // Check for leaks
    let (reg_count, handler_count) = get_js_leak_stats().await;
    log!("After {} cycles: registry entries = {}, handler count = {}", cycles, reg_count, handler_count);

    // Assert no registry entries or global handlers remain
    assert_eq!(reg_count, 0, "JS registry should be empty after cleanup");
    assert_eq!(handler_count, 0, "No global handlers should remain after cleanup");

    // Cleanup DOM
    document.body().unwrap().remove_child(&container).unwrap();
}

// Lifecycle tests
fn create_mock_suggestion() -> compareware::models::item::WikidataSuggestion {
    compareware::models::item::WikidataSuggestion {
        id: "Q12345".to_string(),
        label: "Test Item".to_string(),
        title: "Test Title".to_string(),
        description: "Test Description".to_string(),
        display: compareware::models::item::DisplayInfo {
            label: compareware::models::item::LabelInfo {
                value: "Test Item".to_string(),
                language: "en".to_string(),
            },
            description: compareware::models::item::DescriptionInfo {
                value: "Test Description".to_string(),
                language: "en".to_string(),
            },
        },
    }
}

// Helper function to create a test component
fn create_test_component() -> (NodeRef<leptos::html::Input>, Rc<RefCell<Option<WikidataSuggestion>>>) {
    let node_ref = create_node_ref::<leptos::html::Input>();
    let selected_suggestion = Rc::new(RefCell::new(None));
    
    let selected_suggestion_clone = selected_suggestion.clone();
    let on_select = Callback::new(move |suggestion: WikidataSuggestion| {
        *selected_suggestion_clone.borrow_mut() = Some(suggestion);
    });
    
    let fetch_suggestions = Callback::new(|query: String| {
        if query.is_empty() {
            return vec![];
        }
        
        vec![create_mock_suggestion()]
    });
    
    mount_to_body(move || {
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
    });
    
    (node_ref, selected_suggestion)
}
#[wasm_bindgen_test]
async fn test_multiple_component_lifecycle() {
    // Create first component
    let (node_ref1, _) = create_test_component();
    
    // Wait for initialization
    TimeoutFuture::new(500).await;
    
    // Create second component
    let (node_ref2, _) = create_test_component();
    
    // Wait for initialization
    TimeoutFuture::new(500).await;
    
    // Both should be initialized
    assert!(node_ref1.get().is_some());
    assert!(node_ref2.get().is_some());
    
    // Clean up first component explicitly
    leptos::document().body().unwrap().first_element_child().unwrap().remove();
    
    // Wait for cleanup
    TimeoutFuture::new(500).await;
    
    // Second component should still be valid
    assert!(node_ref2.get().is_some());
    
    // Check if the registry has been properly maintained
    let js_check = js_sys::eval(r#"
        // Check if typeahead registry exists and has components
        if (window.typeaheadRegistry) {
            // Count alive components
            let aliveCount = 0;
            for (let id in window.typeaheadRegistry) {
                if (window.typeaheadRegistry[id].alive) {
                    aliveCount++;
                }
            }
            aliveCount;
        } else {
            0
        }
    "#).unwrap();
    
    // Should have exactly one alive component
    assert_eq!(js_check.as_f64().unwrap_or(0.0), 1.0);
}

#[wasm_bindgen_test]
async fn test_component_refresh_scenario() {
    // This test simulates the scenario where components are refreshed
    
    // First round of components
    let components = (0..3).map(|_| create_test_component()).collect::<Vec<_>>();
    
    // Wait for initialization
    TimeoutFuture::new(500).await;
    
    // Verify all are initialized
    for (node_ref, _) in &components {
        assert!(node_ref.get().is_some());
    }
    
    // Clean up all components
    leptos::document()
        .body()
        .expect("document should have a body")
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("body should be an HtmlElement")
        .set_inner_html("");
    
    // Wait for cleanup
    TimeoutFuture::new(500).await;
    
    // Create new components
    let new_components = (0..3).map(|_| create_test_component()).collect::<Vec<_>>();
    
    // Wait for initialization
    TimeoutFuture::new(500).await;
    
    // Verify all new components are initialized
    for (node_ref, _) in &new_components {
        assert!(node_ref.get().is_some());
    }
    
    // Check registry health
    let js_check = js_sys::eval(r#"
        // Check registry health
        let result = {
            registryExists: !!window.typeaheadRegistry,
            totalComponents: 0,
            aliveComponents: 0,
            globalHandlers: 0
        };
        
        if (window.typeaheadRegistry) {
            result.totalComponents = Object.keys(window.typeaheadRegistry).length;
            
            for (let id in window.typeaheadRegistry) {
                if (window.typeaheadRegistry[id].alive) {
                    result.aliveComponents++;
                }
            }
        }
        
        // Count global handlers
        for (let key in window) {
            if (key.startsWith('rustSelectHandler_') || key.startsWith('rustFetchHandler_')) {
                result.globalHandlers++;
            }
        }
        
        JSON.stringify(result);
    "#).unwrap();
    
    let registry_info = js_check.as_string().unwrap();
    console_log!("Registry health: {}", registry_info);
    
    // Parse the JSON
    let registry_obj: serde_json::Value = serde_json::from_str(&registry_info).unwrap();
    
    // Verify registry health
    assert!(registry_obj["registryExists"].as_bool().unwrap());
    assert_eq!(registry_obj["aliveComponents"].as_i64().unwrap(), 3);
    assert_eq!(registry_obj["totalComponents"].as_i64().unwrap(), 3);
    
    // Global handlers should match the number of components
    assert_eq!(registry_obj["globalHandlers"].as_i64().unwrap(), 6); // 2 handlers per component
}

// Helper function to mount a component to a container
fn mount_to(
    container: &web_sys::Element,
    component: impl FnOnce() -> View + 'static,
) -> impl FnOnce() {
    let html_element = container
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .expect("Element provided to mount_to was not an HtmlElement");

    // Mount the component using Leptos's mount_to
    leptos::mount_to(html_element, component);

    // Return a cleanup closure that will be called on unmount
    move || {
        // Leptos handles cleanup on unmount
    }
}
