use leptos::*;

#[component]
pub fn EditableCell(
    value: String,
    on_input: impl Fn(String) + 'static,
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal(value.clone());

    let handle_input = move |e: web_sys::Event| {
        let new_value = event_target_value(&e);
        set_input_value.set(new_value.clone());
        on_input(new_value);
    };

    view! {
        <input
            type="text"
            value={input_value.get()}
            on:input=handle_input
        />
    }
}
