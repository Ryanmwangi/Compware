use leptos::*;
use crate::models::review::Review;

#[component]
pub fn ReviewsList(reviews: Vec<Review>) -> impl IntoView {
    view! {
        <div>
            <h3>{ "Reviews" }</h3>
            <ul>
                { for review in reviews {
                    view! {
                        <li>{ &review.content }</li>
                    }
                } }
            </ul>
        </div>
    }
}
