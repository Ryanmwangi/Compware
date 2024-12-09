/// Form component for adding a new item.
/// Handles user input for item name, description, and optional tags.
/// Calls `on_submit` when the form is submitted.
use leptos::*;
use leptos_dom::ev::SubmitEvent;

#[component]
pub fn ItemForm(on_submit: Box<dyn Fn(String, String, Vec<(String, String)>)>) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (tags, set_tags) = create_signal(Vec::<(String, String)>::new());

    // Handle form submission.
    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        on_submit(name.get(), description.get(), tags.get().clone());

        // Reset values after submission
        set_name.update(|n| *n = String::new());
        set_description.update(|d| *d = String::new());
        set_tags.update(|t| t.clear());
    };

    view! {
        <form on:submit=handle_submit>
            <input
                type="text"
                placeholder="Name"
                value={name.get()}
                on:input=move |e| set_name.set(event_target_value(&e))
            />
            <textarea
                placeholder="Description"
                value={description.get()}
                on:input=move |e| set_description.set(event_target_value(&e))
            />
            <button type="submit">{ "Add Item" }</button>
        </form>
    }
}