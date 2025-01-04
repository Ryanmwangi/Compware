use leptos::*;
use std::sync::Arc;

#[component]
pub fn EditableCell(
    value: String,
    on_input: impl Fn(String) + 'static,
    key: Arc<String>,
    focused_cell: ReadSignal<Option<String>>,
    set_focused_cell: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal(value.clone());
    let input_ref = NodeRef::<html::Input>::new();

    // Handle input event
    let handle_input = move |e: web_sys::Event| {
        let new_value = event_target_value(&e);
        set_input_value.set(new_value.clone());
        on_input(new_value);
    };

    let handle_focus = {
        let key = Arc::clone(&key);
        move |_| {
            set_focused_cell.set(Some(key.to_string()));
        }
    };

    let handle_blur = move |_| {
        set_focused_cell.set(None);
    };

    create_effect({
        let key = Arc::clone(&key);
        move |_| {
            if let Some(ref current_key) = focused_cell.get() {
                if current_key == key.as_str() {
                    if let Some(input) = input_ref.get() {
                        if web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .active_element()
                            .map_or(true, |el| !el.is_same_node(Some(input.as_ref())))
                        {
                            let _ = input.focus();
                        }
                    }
                }
            }
        }
    });

    view! {
        <input
            type="text"
            prop:value={input_value}
            on:input=handle_input
            on:focus=handle_focus
            on:blur=handle_blur
            node_ref=input_ref
        />
    }
}