use dioxus::prelude::*;
use k8s_openapi::{api::core::v1::Namespace, apimachinery::pkg::apis::meta::v1::ObjectMeta};
use kube::{api::PostParams, Api, Client};
use std::collections::BTreeMap;

const CREATE_FORMS_CSS: Asset = asset!("/assets/styling/create_forms.css");

#[component]
pub fn CreateNamespace() -> Element {
    let client = use_context::<Client>();
    let mut name = use_signal(String::new);
    let mut labels = use_signal(|| Vec::<(String, String)>::new());
    let mut error = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let add_label = move |_| {
        let mut current_labels = labels.read().to_vec();
        current_labels.push((String::new(), String::new()));
        labels.set(current_labels);
    };

    let mut remove_label = move |index: usize| {
        let mut current_labels = labels.read().to_vec();
        current_labels.remove(index);
        labels.set(current_labels);
    };

    let mut update_label = move |index: usize, is_key: bool, value: String| {
        let mut current_labels = labels.read().to_vec();
        if is_key {
            current_labels[index].0 = value;
        } else {
            current_labels[index].1 = value;
        }
        labels.set(current_labels);
    };

    let create_namespace = move |evt: Event<FormData>| {
        evt.prevent_default();
        is_submitting.set(true);
        error.set(String::new());

        let client = client.clone();
        let name = name.read().to_string();
        let labels = labels.read().to_vec();

        spawn(async move {
            let api: Api<Namespace> = Api::all(client);

            let mut label_map = BTreeMap::new();
            for (key, value) in labels {
                if !key.is_empty() && !value.is_empty() {
                    label_map.insert(key, value);
                }
            }

            let namespace = Namespace {
                metadata: ObjectMeta {
                    name: Some(name),
                    labels: Some(label_map),
                    ..Default::default()
                },
                ..Default::default()
            };

            match api.create(&PostParams::default(), &namespace).await {
                Ok(_) => {
                    router().push("/namespaces");
                }
                Err(e) => {
                    error.set(format!("Failed to create namespace: {}", e));
                    is_submitting.set(false);
                }
            }
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CREATE_FORMS_CSS }

        div { class: "create-namespace-container",
            div { class: "create-namespace-header",
                h1 { "Create Namespace" }
            }

            form { class: "create-namespace-form", onsubmit: create_namespace,
                div { class: "form-group",
                    label { "Name" }
                    input {
                        r#type: "text",
                        class: "form-control",
                        value: "{name}",
                        onchange: move |evt| name.set(evt.value().clone()),
                        required: true,
                        placeholder: "my-namespace"
                    }
                    span { class: "help-text", "Name must consist of lowercase letters, numbers, and hyphens" }
                }

                div { class: "form-group",
                    label { "Labels" }
                    div { class: "labels-container",
                        {labels.read().iter().enumerate().map(|(index, (key, value))| (
                            rsx! {
                                div { class: "label-group",
                                    input {
                                        r#type: "text",
                                        class: "form-control",
                                        value: "{key}",
                                        onchange: move |evt| update_label(index, true, evt.value().clone()),
                                        placeholder: "key"
                                    }
                                    input {
                                        r#type: "text",
                                        class: "form-control",
                                        value: "{value}",
                                        onchange: move |evt| update_label(index, false, evt.value().clone()),
                                        placeholder: "value"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn-danger",
                                        onclick: move |_| remove_label(index),
                                        "Remove"
                                    }
                                }
                            }
                        ))}
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-secondary",
                        onclick: add_label,
                        "Add Label"
                    }
                }

                if !error.read().is_empty() {
                    div { class: "error-message",
                        "{error.read()}"
                    }
                }

                div { class: "form-actions",
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: "{is_submitting}",
                        if *is_submitting.read() { "Creating..." } else { "Create Namespace" }
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            use_router().push("/namespaces");
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}
