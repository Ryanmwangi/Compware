use leptos::*;
use leptos::logging::log;
use std::rc::Rc;
use web_sys::FocusEvent;
#[component]
pub fn EditableCell(
    value: String,
    on_input: Rc<dyn Fn(String)>, // Use `Rc` to allow cloning
    #[prop(into)] on_focus: Callback<FocusEvent>,
    #[prop(optional)] key: Option<String>, // Optional `key` prop
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal(value.clone());
    let (has_focus, set_has_focus) = create_signal(false); // Track focus state locally
    let (is_disposed, set_disposed) = create_signal(false); // Track disposal state
    let (is_editing, set_is_editing) = create_signal(false);
    
    // persistent default key value
    let default_key = String::new();
    let key_ref = key.as_ref().unwrap_or(&default_key);

    // Ensure signals aren't updated after disposal
    on_cleanup(move || {
        log!("Component disposed");
        set_disposed.set(true);
    });

    let log_signal_get = move |signal_name: &str| {
        if is_disposed.get() {
            panic!("Attempted to get disposed signal: {}", signal_name);
        }
    };

    let handle_input = move |e: web_sys::Event| {
        let on_input = Rc::clone(&on_input); // Clone `on_input` to use inside the closure
        log_signal_get("input_value");
        if is_disposed.get_untracked() {
            return;
        }
        let new_value = event_target_value(&e);
        set_input_value.set(new_value.clone());
        on_input(new_value);
    };

    let handle_focus = move |ev:FocusEvent| {
        if is_disposed.get() {
            return;
        }
        set_is_editing.set(true);
        set_has_focus.set(true);
        on_focus.call(ev);
    };

    let handle_blur = move |_:FocusEvent| {
        if is_disposed.get() {
            return;
        }
        set_is_editing.set(false);
        set_has_focus.set(false);
    };

    let cell_view = move || {
        if is_editing.get() {
            view! {
                <input
                    type="text"
                    value={input_value.get()}
                    on:input=handle_input.clone()
                    on:focus=handle_focus.clone()
                    on:blur=handle_blur.clone()
                    class={if has_focus.get() { "focused" } else { "not-focused" }}
                />
            }.into_view()
        } else {
            view! {
                <div
                    tabindex="0"
                    on:focus=handle_focus.clone()
                >
                    {input_value.get()}
                </div>
            }.into_view()
        }
    };

    view! {
        <div key={key_ref.clone()}>
            {cell_view}
        </div>
    }
}
