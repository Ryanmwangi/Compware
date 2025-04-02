use leptos::*;
use std::sync::Arc;
use leptos::logging::log;

#[component]
pub fn EditableCell(
    value: String,
    on_input: impl Fn(String) + 'static,
    key: Arc<String>,
    focused_cell: ReadSignal<Option<String>>,
    set_focused_cell: WriteSignal<Option<String>>,
    on_focus: Option<Callback<()>>,
    on_blur: Option<Callback<()>>,
    input_type: InputType,
) -> impl IntoView {
    let input_ref = create_node_ref::<html::Input>();
    let textarea_ref = create_node_ref::<html::Textarea>();
    let (local_value, set_local_value) = create_signal(value.clone());
    let input_type_clone = input_type.clone();
    // Handle input event
    let handle_input = move |e: web_sys::Event| {
        let new_value = match input_type_clone {
            InputType::Text => event_target_value(&e),
            InputType::TextArea => event_target_value(&e),
            InputType::Search => event_target_value(&e),
        };
        log!("Input event: {}", new_value);
        set_local_value.set(new_value.clone());
        on_input(new_value);
    };

    // // Commit the input value on blur or enter
    // let commit_input = move || {
    //     let value = local_value.get();
    //     log!("Committing input: {}", value);
    //     on_input(value);
    // };

    // Focus handling
    let handle_focus = {
        let key = Arc::clone(&key);
        move |_| {
            log!("Focus gained for key: {}", key);
            set_focused_cell.set(Some(key.to_string()));
            if let Some(on_focus) = &on_focus {
                on_focus.call(());
            }
        }
    };

    let handle_blur = move |_| {
        log!("Focus lost");
        set_focused_cell.set(None);
        // commit_input();
        if let Some(on_blur) = &on_blur {
            on_blur.call(());
        }
    };

    // Update input field value when focused cell changes
    create_effect(move |_| {
        if focused_cell.get().as_deref() == Some(key.as_str()) {
            log!("Persisting focus for key: {}", key);
            let input_ref = input_ref.clone();
            let key_clone = key.clone();
            
            // Use requestAnimationFrame for better focus timing
            spawn_local(async move {
                for _ in 0..3 {  // Retry up to 3 times
                    gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                    if let Some(input) = input_ref.get() {
                        let _ = input.focus();
                        if document().active_element().as_ref() == Some(input.as_ref() as &web_sys::Element) {
                            break;
                        }
                    }
                    log!("Focus retry for {}", key_clone);
                }
            });
        }
    });

    view! {
        <div class="editable-cell">
            {match input_type {
                InputType::Text => view! {
                    <input
                        type="text"
                        prop:value=move || local_value.get()
                        on:input=handle_input
                        on:focus=handle_focus
                        on:blur=handle_blur
                        node_ref=input_ref
                        class="editable-cell-input"
                    />
                }.into_view(),
                InputType::TextArea => view! {
                    <textarea
                        prop:value=move || local_value.get()
                        on:input=handle_input
                        on:focus=handle_focus
                        on:blur=handle_blur
                        node_ref=textarea_ref
                        class="editable-cell-input"
                    />
                }.into_view(),
                InputType::Search => view! {
                    <input
                        type="search"
                        prop:value=move || local_value.get()
                        on:input=handle_input
                        on:focus=handle_focus
                        on:blur=handle_blur
                        node_ref=input_ref
                        class="editable-cell-input"
                    />
                }.into_view()
            }}
        </div>
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputType {
    Text,
    TextArea,
    Search
}