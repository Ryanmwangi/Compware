use leptos::*;
use leptos_dom::ev::SubmitEvent;

#[component]
pub fn ItemForm(on_submit: Box<dyn Fn(String, String, Vec<(String, String)>, String, u8)>) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (tags, set_tags) = create_signal(Vec::<(String, String)>::new());
    let (tag_key, set_tag_key) = create_signal(String::new());
    let (tag_value, set_tag_value) = create_signal(String::new());
    let (review, set_review) = create_signal(String::new());
    let (rating, set_rating) = create_signal(5u8); // Default rating to 5

    let add_tag = move |_| {
        if !tag_key.get().is_empty() && !tag_value.get().is_empty() {
            set_tags.update(|t| t.push((tag_key.get(), tag_value.get())));
            set_tag_key.set(String::new());
            set_tag_value.set(String::new());
        }
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        on_submit(
            name.get(),
            description.get(),
            tags.get().clone(),
            review.get(),
            rating.get(),
        );

        // Reset values
        set_name.set(String::new());
        set_description.set(String::new());
        set_tags.set(vec![]);
        set_review.set(String::new());
        set_rating.set(5);
    };

    view! {
        <form on:submit=handle_submit>
            <input type="text" placeholder="Name" on:input=move |e| set_name.set(event_target_value(&e)) />
            <textarea placeholder="Description" on:input=move |e| set_description.set(event_target_value(&e)) />
            <h3>{ "Add Tags" }</h3>
            <input type="text" placeholder="Key" on:input=move |e| set_tag_key.set(event_target_value(&e)) />
            <input type="text" placeholder="Value" on:input=move |e| set_tag_value.set(event_target_value(&e)) />
            <button type="button" on:click=add_tag>{ "Add Tag" }</button>
            <ul>
                {tags.get().iter().map(|(key, value)| view! {
                    <li>{ format!("{}: {}", key, value) }</li>
                }).collect::<Vec<_>>() }
            </ul>
            <h3>{ "Write a Review" }</h3>
            <textarea placeholder="Review" on:input=move |e| set_review.set(event_target_value(&e)) />
            <h3>{ "Rating (1-5)" }</h3>
            <input
                type="number"
                min="1"
                max="5"
                value={rating.get()}
                on:input=move |e| set_rating.set(event_target_value(&e).parse::<u8>().unwrap_or(5))
            />
            <button type="submit">{ "Add Item" }</button>
        </form>
    }
}
