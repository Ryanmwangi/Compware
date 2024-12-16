use leptos::*;

#[component]
pub fn ReviewForm(item_id: String, on_submit: Box<dyn Fn(String) + 'static>) -> impl IntoView {
    let (review_content, set_review_content) = create_signal(String::new());

    let submit_review = move || {
        on_submit(review_content.get());
        set_review_content.set(String::new()); // Clear the textarea after submission
    };

    view! {
        <div>
            <h3>"Submit Review"</h3>
            <textarea
                placeholder="Write your review here"
                prop:value=review_content
                on:input=move |ev| {
                    let input_value = event_target_value(&ev);
                    set_review_content.set(input_value);
                }
            />
            <button on:click=move |_| submit_review()>"Submit Review"</button>
        </div>
    }
}
