use leptos::*;

#[component]
pub fn EditableCell(
    value: String,
    on_input: impl Fn(String) + 'static,
    #[prop(optional)] key: Option<String>, // Optional `key` prop
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal(value.clone());
    let (has_focus, set_has_focus) = create_signal(false); // Track focus state locally

    let handle_input = move |e: web_sys::Event| {
        let new_value = event_target_value(&e);
        set_input_value.set(new_value.clone());
        on_input(new_value);
    };

    let handle_focus = move |_: web_sys::FocusEvent| {
        set_has_focus.set(true);
    };

    let handle_blur = move |_: web_sys::FocusEvent| {
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
