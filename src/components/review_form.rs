use leptos::*;
use crate::models::item::Item;
use leptos::ev::Event;


#[component]
pub fn ReviewForm(item_id: String, on_submit: impl Fn(String) + 'static) -> impl IntoView {
    let (review_content, set_review_content) = create_signal(String::new());

    let submit_review = move |e| {
        on_submit(review_content.get());
    };

    view! {
        <div>
            <h3>{ "Submit Review" }</h3>
            <textarea
                placeholder="Write your review here"
                value={review_content.get()}
                oninput={move |e: Event| set_review_content(e.target().unwrap().value())}
            />
            <button onclick={submit_review}>{ "Submit Review" }</button>
        </div>
    }
}
