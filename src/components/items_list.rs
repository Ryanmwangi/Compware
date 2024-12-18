/// Component to display a list of items.
/// Iterates through the items and renders their name, description, tags, and reviews.
use leptos::*;
use crate::models::item::Item;

#[component]
pub fn ItemsList(items: ReadSignal<Vec<Item>>) -> impl IntoView {
    // Create a signal for selected items
    let (selected_items_signal, set_selected_items) = create_signal(Vec::<usize>::new());

    // Function to toggle selection of an item
    let toggle_selection = move |i: usize| {
        set_selected_items.update(|items| {
            if items.contains(&i) {
                items.retain(|&x| x != i);
            } else {
                items.push(i);
            }
        });
    };

    view! {
        <div>
            <h2>{ "Items" }</h2>
            <ul>
                {move || items.get().iter().enumerate().map(|(i, item)| view! {
                    <li key={i.to_string()}>
                        <label>
                            <input
                                type="checkbox"
                                checked={selected_items_signal.get().contains(&i)}
                                on:change=move |_| toggle_selection(i)
                             />
                             {"Select item for comparison"}
                        </label>
                        <div>
                            <strong>{ &item.name }</strong> - { &item.description }
                        </div>
                        <ul>
                            <h4>{ "Tags:" }</h4>
                            {item.tags.iter().map(|(key, value)| view! {
                                <li>{ key.clone() + ": " + value }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                        <ul>
                            <h4>{ "Reviews:" }</h4>
                            {item.reviews.iter().map(|review| view! {
                                <li>{ format!("Rating: {}/5 - {}", review.rating, review.content) }</li>
                            }).collect::<Vec<_>>()}
                        </ul>
                    </li>
                }).collect::<Vec<_>>() }
            </ul>

            // Comparison Table
            <h2>{ "Comparison Table" }</h2>
            <table>
                <thead>
                    <tr>
                        <th>{ "Item Name" }</th>
                        <th>{ "Description" }</th>
                        <th>{ "Tags" }</th>
                        <th>{ "Reviews" }</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let selected_indices = selected_items_signal.get();
                        selected_indices.iter().map(|&i| {
                            let item = &items.get()[i];
                            view! {
                                <tr key={i.to_string()}>
                                    <td>{ &item.name }</td>
                                    <td>{ &item.description }</td>
                                    <td>
                                        {item.tags.iter().map(|(key, value)| view! {
                                            <span>{ key.clone() + ": " + value + " " }</span>
                                        }).collect::<Vec<_>>()}
                                    </td>
                                    <td>
                                        {item.reviews.iter().map(|review| view! {
                                            <span>{ format!("Rating: {}/5 ", review.rating) }</span>
                                        }).collect::<Vec<_>>()}
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}
