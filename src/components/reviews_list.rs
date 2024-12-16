use leptos::*;
use crate::models::review::Review;

#[component]
pub fn ReviewsList(reviews: Vec<Review>) -> impl IntoView {
    view! {
        <div>
            <h3>{ "Reviews" }</h3>
            <ul>
                { 
                    reviews.into_iter().map(|review| {
                        view! {
                            <li>{ review.content }</li>
                        }
                    }).collect::<Vec<_>>()
                }
            </ul>
        </div>
    }
}
