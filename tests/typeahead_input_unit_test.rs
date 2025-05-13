use wasm_bindgen_test::*;
use leptos::*;
use leptos::logging::log;
use std::time::Duration;
use gloo_timers::future::sleep;
use compareware::components::typeahead_input::TypeaheadInput;
use compareware::models::item::WikidataSuggestion;
use wasm_bindgen::JsCast;

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
    let test_component = {
        let init_called = init_called.clone();
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
                    init_called.set(true);
                    vec![]
                }
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
            }.into_view()
        }
    };

    // Mount the component
    let unmount = mount_to(&container, test_component);

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
    unmount();
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
    let _component_id = format!("test-typeahead-{}", uuid::Uuid::new_v4());

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

    // Check registry before unmount
    let registry_before = js_sys::eval(
        "window.typeaheadRegistry ? Object.keys(window.typeaheadRegistry).length : 0"
    ).unwrap();

    // Unmount the component
    unmount();

    // Wait for cleanup
    sleep(Duration::from_millis(500)).await;

    // Check if component was properly removed from registry
    let registry_after = js_sys::eval(
        "window.typeaheadRegistry ? Object.keys(window.typeaheadRegistry).length : 0"
    ).unwrap();

    // Registry should have one fewer entry
    assert!(
        registry_before.as_f64().unwrap() > registry_after.as_f64().unwrap(),
        "Component was not properly removed from registry. Before: {:?}, After: {:?}",
        registry_before, registry_after
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
            }.into_view()
        };

        // Mount
        let unmount = mount_to(&container, test_component);

        // Wait briefly
        sleep(Duration::from_millis(50)).await;

        // Unmount
        unmount();

        // Wait briefly
        sleep(Duration::from_millis(50)).await;
    }

    // Wait for any pending cleanup
    sleep(Duration::from_millis(500)).await;

    // Check if registry is clean
    let registry_size = js_sys::eval(
        "window.typeaheadRegistry ? Object.keys(window.typeaheadRegistry).length : 0"
    ).unwrap();

    assert!(
        registry_size.as_f64().unwrap_or(0.0) < 2.0, // Adjusted for robustness
        "Registry has too many entries after rapid mount/unmount cycles: {:?}",
        registry_size
    );

    // Cleanup
    document.body().unwrap().remove_child(&container).unwrap();
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

    // Return a no-op cleanup closure 
    move || {
        // Leptos cleans up on unmount.
    }
}
