use wasm_bindgen::prelude::*;

/// This module provides mock implementations for JavaScript dependencies
/// that are used in the TypeaheadInput component.

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// JavaScript functions for mocking Bloodhound
#[wasm_bindgen]
extern "C" {
    /// Injects the Bloodhound mock into the window object
    #[wasm_bindgen(js_name = setup_bloodhound_mock)]
    #[doc(hidden)]
    fn _setup_bloodhound_mock() -> bool;

    /// Gets the size of the typeahead registry
    #[wasm_bindgen(js_name = get_registry_size)]
    #[doc(hidden)]
    fn _get_registry_size() -> usize;

    /// Cleans up a specific component from the registry
    #[wasm_bindgen(js_name = cleanup_typeahead_registry)]
    #[doc(hidden)]
    fn _cleanup_typeahead_registry(component_id: &str) -> bool;

    /// Cleans up the entire typeahead registry
    #[wasm_bindgen(js_name = cleanup_all_typeahead_registry)]
    #[doc(hidden)]
    fn _cleanup_all_typeahead_registry() -> usize;
}

/// Injects the Bloodhound mock into the window object
pub fn setup_bloodhound_mock() -> bool {
    #[wasm_bindgen(inline_js = r#"
    export function setup_bloodhound_mock() {
        // Create a mock Bloodhound constructor
        window.Bloodhound = function(options) {
            this.options = options || {};
            this.initialized = false;
            
            // Store the remote function if provided in options
            if (options && options.remote && typeof options.remote.transport === 'function') {
                this.transportFn = options.remote.transport;
            }
            
            console.log("[MOCK] Bloodhound constructor called with options:", JSON.stringify(options));
        };
        
        // Add initialize method
        window.Bloodhound.prototype.initialize = function(reinitialize) {
            this.initialized = true;
            console.log("[MOCK] Bloodhound initialized, reinitialize:", reinitialize);
            return true;
        };
        
        // Add get method (returns suggestions)
        window.Bloodhound.prototype.get = function(query, cb) {
            console.log("[MOCK] Bloodhound get called with query:", query);
            
            // If we have a transport function, use it
            if (this.transportFn) {
                this.transportFn(query, 
                    // sync callback
                    function(suggestions) {
                        console.log("[MOCK] Bloodhound sync callback with suggestions:", JSON.stringify(suggestions));
                        cb(suggestions);
                    },
                    // async callback
                    function(suggestions) {
                        console.log("[MOCK] Bloodhound async callback with suggestions:", JSON.stringify(suggestions));
                        cb(suggestions);
                    }
                );
            } else {
                // Return empty results if no transport function
                cb([]);
            }
        };
        
        // Setup typeahead registry if it doesn't exist
        if (!window.typeaheadRegistry) {
            window.typeaheadRegistry = {};
        }
        
        console.log("[MOCK] Bloodhound mock setup complete");
        return true;
    }
    "#)]
    extern "C" {
        fn setup_bloodhound_mock() -> bool;
    }
    
    setup_bloodhound_mock()
}

/// Gets the size of the typeahead registry
pub fn get_registry_size() -> usize {
    #[wasm_bindgen(inline_js = r#"
    export function get_registry_size() {
        if (window.typeaheadRegistry) {
            return Object.keys(window.typeaheadRegistry).length;
        }
        return 0;
    }
    "#)]
    extern "C" {
        fn get_registry_size() -> usize;
    }
    
    get_registry_size()
}

/// Cleans up a specific component from the registry
pub fn cleanup_typeahead_registry(component_id: &str) -> bool {
    #[wasm_bindgen(inline_js = r#"
    export function cleanup_typeahead_registry(component_id) {
        if (window.typeaheadRegistry && window.typeaheadRegistry[component_id]) {
            delete window.typeaheadRegistry[component_id];
            console.log("[MOCK] Cleaned up registry for component:", component_id);
            return true;
        }
        return false;
    }
    "#)]
    extern "C" {
        fn cleanup_typeahead_registry(component_id: &str) -> bool;
    }
    
    cleanup_typeahead_registry(component_id)
}

/// Cleans up the entire typeahead registry
pub fn cleanup_all_typeahead_registry() -> usize {
    #[wasm_bindgen(inline_js = r#"
    export function cleanup_all_typeahead_registry() {
        if (window.typeaheadRegistry) {
            const count = Object.keys(window.typeaheadRegistry).length;
            window.typeaheadRegistry = {};
            console.log("[MOCK] Cleaned up entire registry, removed components:", count);
            return count;
        }
        return 0;
    }
    "#)]
    extern "C" {
        fn cleanup_all_typeahead_registry() -> usize;
    }
    
    cleanup_all_typeahead_registry()
}