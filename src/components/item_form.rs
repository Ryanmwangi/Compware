use leptos::*;
use leptos_dom::ev::SubmitEvent;

#[component]
pub fn ItemForm(
    on_submit: Box<dyn Fn(String, String, Vec<(String, String)>, String) + 'static>
) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (tags, set_tags) = create_signal(Vec::<(String, String)>::new());
    let (tag_key, set_tag_key) = create_signal(String::new());
    let (tag_value, set_tag_value) = create_signal(String::new());
    let (review, set_review) = create_signal(String::new());

    // Handle adding a new tag
    let add_tag = move |_| {
        if !tag_key.get().is_empty() && !tag_value.get().is_empty() {
            set_tags.update(|t| t.push((tag_key.get(), tag_value.get())));
            set_tag_key.set(String::new());
            set_tag_value.set(String::new());
        }
    };

    // Handle form submission.
    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        on_submit(
            name.get(),              // Item name
            description.get(),       // Item description
            tags.get().clone(),      // Tags
            review.get(),            // Review
        );

        // Reset values after submission
        set_name.set(String::new());
        set_description.set(String::new());
        set_tags.set(vec![]);
        set_review.set(String::new());
    };

    view! {
        <form on:submit=handle_submit>
            // Item Name Input
            <input
                type="text"
                placeholder="Name"
                value={name.get()}
                on:input=move |e| set_name.set(event_target_value(&e))
            />

            // Item Description Input
            <textarea
                placeholder="Description"
                value={description.get()}
                on:input=move |e| set_description.set(event_target_value(&e))
            />

            // Tags Section
            <div>
                <h3>{ "Add Tags" }</h3>
                <input
                    type="text"
                    placeholder="Key"
                    value={tag_key.get()}
                    on:input=move |e| set_tag_key.set(event_target_value(&e))
                />
                <input
                    type="text"
                    placeholder="Value"
                    value={tag_value.get()}
                    on:input=move |e| set_tag_value.set(event_target_value(&e))
                />
                <button type="button" on:click=add_tag>{ "Add Tag" }</button>
            </div>
            <ul>
                {tags.get().iter().map(|(key, value)| view! {
                    <li>{ format!("{}: {}", key, value) }</li>
                }).collect::<Vec<_>>() }
            </ul>

            // Review Input
            <div>
                <h3>{ "Review" }</h3>
                <textarea
                    placeholder="Write your review here"
                    value={review.get()}
                    on:input=move |e| set_review.set(event_target_value(&e))
                />
            </div>

            // Submit Button
            <button type="submit">{ "Add Item with Review" }</button>
        </form>
    }
}
