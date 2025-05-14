use wasm_bindgen::prelude::*;

/// This module provides a mock implementation of jQuery for testing
/// the TypeaheadInput component without requiring the actual jQuery library.

/// Injects a minimal jQuery mock into the window object
pub fn setup_jquery_mock() -> bool {
    #[wasm_bindgen(inline_js = r#"
    export function setup_jquery_mock() {
        // Create a minimal jQuery mock
        window.$ = function(selector) {
            console.log("[MOCK JQUERY] Selector:", selector);
            
            // Return a mock jQuery object with common methods
            return {
                typeahead: function(action, options) {
                    console.log("[MOCK JQUERY] Typeahead called with action:", action, "options:", JSON.stringify(options));
                    return this;
                },
                on: function(event, handler) {
                    console.log("[MOCK JQUERY] Registered event handler for:", event);
                    return this;
                },
                val: function(value) {
                    if (value === undefined) {
                        console.log("[MOCK JQUERY] Getting value");
                        return "";
                    } else {
                        console.log("[MOCK JQUERY] Setting value to:", value);
                        return this;
                    }
                },
                trigger: function(event) {
                    console.log("[MOCK JQUERY] Triggered event:", event);
                    return this;
                }
            };
        };
        
        // Add jQuery.fn as an alias for jQuery prototype
        window.$.fn = window.$.prototype;
        
        console.log("[MOCK] jQuery mock setup complete");
        return true;
    }
    "#)]
    extern "C" {
        fn setup_jquery_mock() -> bool;
    }
    
    setup_jquery_mock()
}