use leptos::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
struct WikidataResult {
    id: String,
    label: String,
    description: Option<String>,
}

#[component]
pub fn WikidataLookup(
    query: String,
    on_select: impl Fn(WikidataResult) + 'static,
) -> impl IntoView {
    let (suggestions, set_suggestions) = create_signal(Vec::new());

    let fetch_suggestions = move |query: String| {
        spawn_local(async move {
            if query.is_empty() {
                set_suggestions(Vec::new());
                return;
            }
            let url = format!("https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&limit=5&format=json&origin=*", query);
            if let Ok(response) = reqwest::get(&url).await {
                if let Ok(data) = response.json::<WikidataResponse>().await {
                    set_suggestions(data.search);
                }
            }
        });
    };

    create_effect(move || {
        fetch_suggestions(query.clone());
    });

    view! {
        <ul>
            {suggestions.get().iter().map(|suggestion| {
                view! {
                    <li on:click=move |_| on_select(suggestion.clone())>
                        {format!("{} - {}", suggestion.label, suggestion.description.clone().unwrap_or_default())}
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ul>
    }
}
