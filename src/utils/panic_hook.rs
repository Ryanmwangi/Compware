use std::panic;
use leptos::logging::log;
use wasm_bindgen::prelude::*;

/// Sets up a custom panic hook that provides more context for Leptos owner disposal panics
pub fn set_custom_panic_hook() {
    let original_hook = panic::take_hook();
    
    panic::set_hook(Box::new(move |panic_info| {
        // Call the original hook first
        original_hook(panic_info);
        
        // Extract panic message
        let message = if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic".to_string()
        };
        
        // Check if this is an owner disposal panic
        if message.contains("OwnerDisposed") {
            log!("[PANIC] Leptos owner disposal detected. This usually happens when:");
            log!("[PANIC] 1. A component has been unmounted but JavaScript is still calling into Rust");
            log!("[PANIC] 2. An effect or signal update is running after the component is gone");
            log!("[PANIC] 3. A closure or callback is being called after cleanup");
            
            // Log current component registry state
            let js_code = r#"
                if (window.typeaheadRegistry) {
                    console.log('[PANIC] Current typeahead registry:', 
                        Object.keys(window.typeaheadRegistry).map(id => ({
                            id,
                            alive: window.typeaheadRegistry[id].alive,
                            initialized: window.typeaheadRegistry[id].initialized
                        }))
                    );
                } else {
                    console.log('[PANIC] No typeahead registry found');
                }
            "#;
            
            let _ = js_sys::eval(js_code);
        }
    }));
}

/// Call in main.rs or app initialization
pub fn init() {
    log!("[PANIC_HOOK] Setting up custom panic hook");
    set_custom_panic_hook();
    log!("[PANIC_HOOK] Custom panic hook set up successfully");
}