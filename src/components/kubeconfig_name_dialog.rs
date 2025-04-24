#![allow(non_snake_case)] // Allow non-snake_case for component names

use dioxus::prelude::*;

const DIALOG_CSS: Asset = asset!("/assets/styling/dialog.css");

#[derive(Props, PartialEq, Clone)]
pub struct KubeconfigNameDialogProps {
    pub original_filename: String,
    pub on_close: EventHandler<Option<String>>,
}

pub fn KubeconfigNameDialog(props: KubeconfigNameDialogProps) -> Element {
    let mut input_name = use_signal(|| {
        // Suggest a name based on the original filename, removing common extensions
        props.original_filename
            .trim_end_matches(".yaml")
            .trim_end_matches(".yml")
            .trim_end_matches(".kubeconfig")
            .to_string()
    });

    rsx! {
        document::Link { rel: "stylesheet", href: DIALOG_CSS }
        div { class: "dialog-overlay", // Modal overlay
            div { class: "dialog-box", // Dialog container
                h3 { "Name Kubeconfig Context" }
                p { "Original file: {props.original_filename}" }
                div { class: "dialog-input-group",
                    label { r#for: "kubeconfig-name-input", "Context Name:" }
                    input {
                        id: "kubeconfig-name-input",
                        r#type: "text",
                        value: "{input_name}",
                        // Use oninput for immediate updates to the signal
                        oninput: move |evt| input_name.set(evt.value()),
                        // Allow submitting with Enter key
                        onkeydown: move |evt| {
                            if evt.key() == Key::Enter && !input_name.read().is_empty() {
                                props.on_close.call(Some(input_name()));
                            }
                        }
                    }
                }
                div { class: "dialog-buttons",
                    button {
                        class: "dialog-button cancel",
                        onclick: move |_| props.on_close.call(None),
                        "Cancel"
                    }
                    button {
                        class: "dialog-button ok",
                        // Disable OK button if input is empty
                        disabled: input_name.read().is_empty(),
                        onclick: move |_| {
                            if !input_name.read().is_empty() {
                                props.on_close.call(Some(input_name()));
                            }
                        },
                        "OK"
                    }
                }
            }
        }
    }
}

