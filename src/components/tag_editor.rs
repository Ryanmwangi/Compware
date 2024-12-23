use leptos::*;
use std::sync::{Arc, Mutex};



#[component]
pub fn TagEditor(
    tags: Vec<(String, String)>,
    on_add: impl Fn(String, String) + 'static,
    on_remove: Arc<Mutex<dyn FnMut(usize) + Send + Sync>>,
) -> impl IntoView {
    let (key, set_key) = create_signal(String::new());
    let (value, set_value) = create_signal(String::new());

    let add_tag = move |_| {
        if !key.get().is_empty() && !value.get().is_empty() {
            on_add(key.get(), value.get());
            set_key.set(String::new());
            set_value.set(String::new());
        }
    };

    view! {
        <div>
            <ul>
                {tags.iter().enumerate().map(|(index, (k, v))| {
                    let on_remove = on_remove.clone();
                    view! {
                        <li>
                            {format!("{}: {}", k, v)}
                            <button on:click=move |_| {
                                let mut on_remove = on_remove.lock().unwrap();
                                on_remove(index);
                            }>
                            { "Remove" }
                            </button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
            <input
                placeholder="Key"
                value={key.get()}
                on:input=move |e| set_key.set(event_target_value(&e))
            />
            <input
                placeholder="Value"
                value={value.get()}
                on:input=move |e| set_value.set(event_target_value(&e))
            />
            <button on:click=add_tag>{ "Add Tag" }</button>
        </div>
    }
}
