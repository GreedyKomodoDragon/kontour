use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct SearchInputProps {
    query: String,
    on_change: EventHandler<String>,
}

#[component]
pub fn SearchInput(props: SearchInputProps) -> Element {
    rsx! {
        div { class: "search-container",
            input {
                class: "search-input",
                r#type: "text",
                placeholder: "Search...",
                value: "{props.query}",
                oninput: move |evt| props.on_change.call(evt.value().clone())
            }
        }
    }
}
