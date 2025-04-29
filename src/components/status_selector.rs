use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct StatusSelectorProps {
    selected_status: String,
    on_change: EventHandler<String>,
}

#[component]
pub fn StatusSelector(props: StatusSelectorProps) -> Element {
    let statuses = vec!["All", "Running", "Pending", "Failed", "Succeeded"];

    rsx! {
        select {
            class: "status-select",
            value: "{props.selected_status}",
            onchange: move |evt| {
                props.on_change.call(evt.value());
            },
            {statuses.iter().map(|status| {
                rsx! {
                    option {
                        value: "{status}",
                        "{status}"
                    }
                }
            })}
        }
    }
}
