use leptos::*;
use leptos::logging::log;
use web_sys::FocusEvent;
#[component]
pub fn EditableCell(
    value: String,
    on_input: impl Fn(String) + 'static,
    #[prop(into)] on_focus: Callback<FocusEvent>,
    #[prop(optional)] key: Option<String>, // Optional `key` prop
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal(value.clone());
    let (has_focus, set_has_focus) = create_signal(false); // Track focus state locally

    let (is_disposed, set_disposed) = create_signal(false); // Track disposal state

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
        set_has_focus.set(true);
        on_focus.call(ev);
    };

    let handle_blur = move |_:FocusEvent| {
        if is_disposed.get() {
            return;
        }
        set_has_focus.set(false);
    };

    // Use key to force updates only when necessary
    let _key = key.unwrap_or_default();

    view! {
        <input
            type="text"
            value={input_value.get()}
            on:input=handle_input
            on:focus=handle_focus
            on:blur=handle_blur
            class={if has_focus.get() { "focused" } else { "not-focused" }}
        />
    }
}
