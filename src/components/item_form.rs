use leptos::*;

#[component]
pub fn ItemForm(on_submit: Box<dyn Fn(String, String, Vec<(String, String)>)>) -> impl IntoView {
    let name = create_signal(String::new());
    let description = create_signal(String::new());
    let tags = create_signal(vec![]);

    let handle_submit = move |_| {
        on_submit(name.get().clone(), description.get().clone(), tags.get().clone());
        name.set(String::new());
        description.set(String::new());
        tags.set(vec![]);
    };

    view! {
        <form on:submit=handle_submit>
            <input type="text" placeholder="Name" value=name.clone() on:input=move |e| name.set(event_target_value(&e)) />
            <textarea placeholder="Description" value=description.clone() on:input=move |e| description.set(event_target_value(&e)) />
            <button type="submit">{ "Add Item" }</button>
        </form>
    }
}